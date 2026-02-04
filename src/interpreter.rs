use crate::ast::{Expression, Program, Statement};
use crate::env::{Environment, Object};
use crate::token::Token;
use std::cell::RefCell;
use std::rc::Rc;

pub struct Interpreter {
    // env: Rc<RefCell<Environment>>,
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {}
    }

    pub fn eval_program(&mut self, program: &Program, env: Rc<RefCell<Environment>>) -> Object {
        let mut result = Object::Null;

        for stmt in &program.statements {
            result = self.eval_statement(stmt, Rc::clone(&env));

            if let Object::ReturnValue(val) = result {
                return *val;
            }
            if let Object::Error(_) = result {
                return result;
            }
        }

        result
    }

    fn eval_block(&mut self, statements: &Vec<Statement>, env: Rc<RefCell<Environment>>) -> Object {
        let mut result = Object::Null;

        for stmt in statements {
            result = self.eval_statement(stmt, Rc::clone(&env));

            if let Object::ReturnValue(_) = result {
                return result;
            }
            if let Object::Error(_) = result {
                return result;
            }
        }

        result
    }

    fn eval_statement(&mut self, stmt: &Statement, env: Rc<RefCell<Environment>>) -> Object {
        match stmt {
            Statement::Expression(expr) => self.eval_expression(expr, env),
            Statement::Return(expr) => {
                let val = self.eval_expression(expr, env);
                if self.is_error(&val) {
                    return val;
                }
                Object::ReturnValue(Box::new(val))
            }
            Statement::Let { name, value } => {
                let val = self.eval_expression(value, Rc::clone(&env));
                if self.is_error(&val) {
                    return val;
                }
                env.borrow_mut().set(name.clone(), val)
            }
            Statement::Block(stmts) => self.eval_block(stmts, env),
            Statement::If {
                condition,
                consequence,
                alternative,
            } => {
                let cond = self.eval_expression(condition, Rc::clone(&env));
                if self.is_error(&cond) {
                    return cond;
                }

                if self.is_truthy(&cond) {
                    self.eval_statement(consequence, env)
                } else if let Some(alt) = alternative {
                    self.eval_statement(alt, env)
                } else {
                    Object::Null
                }
            }
            Statement::While { condition, body } => {
                loop {
                    let cond = self.eval_expression(condition, Rc::clone(&env));
                    if self.is_error(&cond) {
                        return cond;
                    }

                    if !self.is_truthy(&cond) {
                        break;
                    }

                    let result = self.eval_statement(body, Rc::clone(&env));
                    // Handle return inside while?
                    match result {
                        Object::ReturnValue(_) | Object::Error(_) => return result,
                        _ => {}
                    }
                }
                Object::Null
            }
            Statement::Function { name, params, body } => {
                let func = Object::Function(params.clone(), body.clone(), Rc::clone(&env));
                env.borrow_mut().set(name.clone(), func)
            }
        }
    }

    fn eval_expression(&mut self, expr: &Expression, env: Rc<RefCell<Environment>>) -> Object {
        match expr {
            Expression::Integer(val) => Object::Integer(*val),
            Expression::String(val) => Object::String(val.clone()),
            Expression::Boolean(val) => Object::Boolean(*val),
            Expression::Identifier(name) => match env.borrow().get(name) {
                Some(val) => val,
                None => Object::Error(format!("identifier not found: {}", name)),
            },
            Expression::Prefix { operator, right } => {
                let right_val = self.eval_expression(right, env);
                if self.is_error(&right_val) {
                    return right_val;
                }
                self.eval_prefix_expression(operator, right_val)
            }
            Expression::Infix {
                left,
                operator,
                right,
            } => {
                let left_val = self.eval_expression(left, Rc::clone(&env));
                if self.is_error(&left_val) {
                    return left_val;
                }

                let right_val = self.eval_expression(right, env);
                if self.is_error(&right_val) {
                    return right_val;
                }

                self.eval_infix_expression(operator, left_val, right_val)
            }
            Expression::Call {
                function,
                arguments,
            } => {
                let func = self.eval_expression(function, Rc::clone(&env));
                if self.is_error(&func) {
                    return func;
                }

                let mut args = vec![];
                for arg in arguments {
                    let val = self.eval_expression(arg, Rc::clone(&env));
                    if self.is_error(&val) {
                        return val;
                    }
                    args.push(val);
                }

                if let Object::Function(params, body, func_env) = func {
                    if params.len() != args.len() {
                        return Object::Error(format!(
                            "wrong number of arguments: want={}, got={}",
                            params.len(),
                            args.len()
                        ));
                    }

                    // New environment!
                    let mut enclosed = Environment::new_enclosed(func_env);
                    for (param, arg) in params.iter().zip(args) {
                        enclosed.set(param.clone(), arg);
                    }

                    let result = self.eval_statement(&body, Rc::new(RefCell::new(enclosed)));
                    // Unwrap return value if present
                    if let Object::ReturnValue(val) = result {
                        *val
                    } else {
                        result
                    }
                } else if let Object::Builtin(func) = func {
                    func(args)
                } else {
                    Object::Error(format!("not a function: {:?}", func))
                }
            }
        }
    }

    fn eval_prefix_expression(&self, operator: &Token, right: Object) -> Object {
        match operator {
            Token::Minus => match right {
                Object::Integer(val) => Object::Integer(-val),
                _ => Object::Error(format!("unknown operator: -{:?}", right)),
            },
            _ => Object::Error(format!("unknown operator: {:?}{:?}", operator, right)),
        }
    }

    fn eval_infix_expression(&self, operator: &Token, left: Object, right: Object) -> Object {
        match (left, right) {
            (Object::Integer(l), Object::Integer(r)) => match operator {
                Token::Plus => Object::Integer(l + r),
                Token::Minus => Object::Integer(l - r),
                Token::Asterisk => Object::Integer(l * r),
                Token::Slash => Object::Integer(l / r),
                Token::LessThan => Object::Boolean(l < r),
                Token::GreaterThan => Object::Boolean(l > r),
                Token::Equal => Object::Boolean(l == r),
                Token::NotEqual => Object::Boolean(l != r),
                _ => Object::Error(format!("unknown operator: INTEGER {:?} INTEGER", operator)),
            },
            (Object::Boolean(l), Object::Boolean(r)) => match operator {
                Token::Equal => Object::Boolean(l == r),
                Token::NotEqual => Object::Boolean(l != r),
                _ => Object::Error(format!("unknown operator: BOOLEAN {:?} BOOLEAN", operator)),
            },
            (l, r) => match operator {
                Token::Equal => Object::Boolean(l == r),
                Token::NotEqual => Object::Boolean(l != r),
                _ => Object::Error(format!("type mismatch: {:?} {:?} {:?}", l, operator, r)),
            },
        }
    }

    fn is_truthy(&self, obj: &Object) -> bool {
        match obj {
            Object::Null => false,
            Object::Boolean(val) => *val,
            Object::Integer(val) => *val != 0,
            _ => true,
        }
    }

    fn is_error(&self, obj: &Object) -> bool {
        matches!(obj, Object::Error(_))
    }
}
