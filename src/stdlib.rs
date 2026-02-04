use crate::env::{Environment, FileHandle, Object};
use std::cell::RefCell;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::rc::Rc;

fn format_output(args: Vec<Object>) -> Result<String, String> {
    if args.is_empty() {
        return Ok(String::new());
    }

    let fmt_str = match &args[0] {
        Object::String(s) => s.clone(),
        _ => return Ok(args.iter().map(|a| a.inspect()).collect::<String>()), // Fallback to join if not string fmt
    };

    let fmt_args = &args[1..];
    let mut out = String::new();
    let mut arg_idx = 0;
    let mut chars = fmt_str.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '%' {
            if let Some(&next_c) = chars.peek() {
                match next_c {
                    's' | 'd' => {
                        chars.next(); // consume specifier
                        if arg_idx < fmt_args.len() {
                            out.push_str(&fmt_args[arg_idx].inspect());
                            arg_idx += 1;
                        } else {
                            out.push('%');
                            out.push(next_c);
                        }
                    }
                    '%' => {
                        chars.next();
                        out.push('%');
                    }
                    _ => {
                        out.push('%');
                    }
                }
            } else {
                out.push('%');
            }
        } else {
            out.push(c);
        }
    }
    Ok(out)
}

pub fn register_stdlib(env: Rc<RefCell<Environment>>) {
    let mut env_mut = env.borrow_mut();

    // puts(str)
    env_mut.set(
        "puts".to_string(),
        Object::Builtin(|args| {
            if args.len() != 1 {
                return Object::Error(format!("puts expected 1 argument, got {}", args.len()));
            }
            println!("{}", args[0].inspect());
            Object::Null
        }),
    );

    // putchar(char)
    env_mut.set(
        "putchar".to_string(),
        Object::Builtin(|args| {
            if args.len() != 1 {
                return Object::Error("putchar expected 1 argument".to_string());
            }
            let s = args[0].inspect();
            if let Some(c) = s.chars().next() {
                print!("{}", c);
            }
            Object::Null
        }),
    );

    // printf(fmt, ...)
    env_mut.set(
        "printf".to_string(),
        Object::Builtin(|args| match format_output(args) {
            Ok(s) => {
                print!("{}", s);
                Object::Null
            }
            Err(e) => Object::Error(e),
        }),
    );

    // sprintf(fmt, ...) -> String
    env_mut.set(
        "sprintf".to_string(),
        Object::Builtin(|args| match format_output(args) {
            Ok(s) => Object::String(s),
            Err(e) => Object::Error(e),
        }),
    );

    // fopen(path, mode)
    env_mut.set(
        "fopen".to_string(),
        Object::Builtin(|args| {
            if args.len() != 2 {
                return Object::Error(format!("fopen expected 2 arguments, got {}", args.len()));
            }
            let path = match &args[0] {
                Object::String(s) => s,
                _ => {
                    return Object::Error("fopen first argument must be a string path".to_string());
                }
            };
            let mode = match &args[1] {
                Object::String(s) => s,
                _ => {
                    return Object::Error(
                        "fopen second argument must be a string mode".to_string(),
                    );
                }
            };

            let file = if mode == "r" {
                File::open(path)
            } else if mode == "w" {
                File::create(path)
            } else {
                File::open(path) // Default read
            };

            match file {
                Ok(f) => Object::File(Rc::new(RefCell::new(FileHandle {
                    file: f,
                    eof: false,
                    error: false,
                }))),
                Err(e) => Object::Error(format!("fopen failed: {}", e)),
            }
        }),
    );

    // fclose(file)
    env_mut.set("fclose".to_string(), Object::Builtin(|_args| Object::Null));

    // fputs(str, file)
    env_mut.set(
        "fputs".to_string(),
        Object::Builtin(|args| {
            if args.len() != 2 {
                return Object::Error("fputs expected 2 arguments".to_string());
            }
            let content = match &args[0] {
                Object::String(s) => s,
                _ => return Object::Error("fputs first arg must be string".to_string()),
            };

            match &args[1] {
                Object::File(handle) => {
                    let mut fh = handle.borrow_mut();
                    if let Err(e) = write!(fh.file, "{}", content) {
                        fh.error = true;
                        Object::Error(format!("fputs failed: {}", e))
                    } else {
                        Object::Null
                    }
                }
                _ => Object::Error("fputs second arg must be file".to_string()),
            }
        }),
    );

    // fputc(char, file)
    env_mut.set(
        "fputc".to_string(),
        Object::Builtin(|args| {
            if args.len() != 2 {
                return Object::Error("fputc expected 2 arguments".to_string());
            }
            let c_str = args[0].inspect();
            let c = if let Some(ch) = c_str.chars().next() {
                ch
            } else {
                return Object::Null;
            };

            match &args[1] {
                Object::File(handle) => {
                    let mut fh = handle.borrow_mut();
                    if let Err(e) = write!(fh.file, "{}", c) {
                        fh.error = true;
                        Object::Error(format!("fputc failed: {}", e))
                    } else {
                        Object::Null
                    }
                }
                _ => Object::Error("fputc arg must be file".to_string()),
            }
        }),
    );

    // fprintf(file, fmt, ...)
    env_mut.set(
        "fprintf".to_string(),
        Object::Builtin(|args| {
            if args.len() < 2 {
                return Object::Error("fprintf expected at least file and fmt".to_string());
            }

            let file_obj = &args[0];
            // Need to extract other args for formatting
            // args[1] is fmt.

            if let Object::File(handle) = file_obj {
                let fmt_args = args[1..].to_vec(); // clone args
                match format_output(fmt_args) {
                    Ok(s) => {
                        let mut fh = handle.borrow_mut();
                        if let Err(_) = write!(fh.file, "{}", s) {
                            fh.error = true;
                            Object::Error("write error".to_string())
                        } else {
                            Object::Null
                        }
                    }
                    Err(e) => Object::Error(e),
                }
            } else {
                Object::Error("fprintf first arg must be file".to_string())
            }
        }),
    );

    // fgets(file)
    env_mut.set(
        "fgets".to_string(),
        Object::Builtin(|args| {
            if args.len() != 1 {
                return Object::Error("fgets expected 1 argument".to_string());
            }
            match &args[0] {
                Object::File(handle) => {
                    let mut fh = handle.borrow_mut();
                    let mut line = String::new();
                    let mut buf = [0; 1];
                    loop {
                        match fh.file.read(&mut buf) {
                            Ok(0) => {
                                fh.eof = true;
                                break;
                            }
                            Ok(_) => {
                                let c = buf[0] as char;
                                line.push(c);
                                if c == '\n' {
                                    break;
                                }
                            }
                            Err(e) => {
                                fh.error = true;
                                return Object::Error(format!("fgets error: {}", e));
                            }
                        }
                    }
                    if line.is_empty() && fh.eof {
                        Object::Null
                    } else {
                        Object::String(line)
                    }
                }
                _ => Object::Error("fgets arg must be file".to_string()),
            }
        }),
    );

    // fgetc(file)
    env_mut.set(
        "fgetc".to_string(),
        Object::Builtin(|args| {
            if args.len() != 1 {
                return Object::Error("fgetc expected 1 arg".to_string());
            }
            match &args[0] {
                Object::File(handle) => {
                    let mut fh = handle.borrow_mut();
                    let mut buf = [0; 1];
                    match fh.file.read(&mut buf) {
                        Ok(0) => {
                            fh.eof = true;
                            Object::Null
                        }
                        Ok(_) => Object::String((buf[0] as char).to_string()),
                        Err(_) => {
                            fh.error = true;
                            Object::Null
                        }
                    }
                }
                _ => Object::Error("fgetc arg must be file".to_string()),
            }
        }),
    );

    // feof(file)
    env_mut.set(
        "feof".to_string(),
        Object::Builtin(|args| {
            if args.len() != 1 {
                return Object::Error("feof expected 1 arg".to_string());
            }
            match &args[0] {
                Object::File(handle) => Object::Boolean(handle.borrow().eof),
                _ => Object::Error("feof arg must be file".to_string()),
            }
        }),
    );

    // ferror(file)
    env_mut.set(
        "ferror".to_string(),
        Object::Builtin(|args| {
            if args.len() != 1 {
                return Object::Error("ferror expected 1 arg".to_string());
            }
            match &args[0] {
                Object::File(handle) => Object::Boolean(handle.borrow().error),
                _ => Object::Error("ferror arg must be file".to_string()),
            }
        }),
    );

    // ftell(file)
    env_mut.set(
        "ftell".to_string(),
        Object::Builtin(|args| {
            if args.len() != 1 {
                return Object::Error("ftell expected 1 arg".to_string());
            }
            match &args[0] {
                Object::File(handle) => match handle.borrow_mut().file.stream_position() {
                    Ok(pos) => Object::Integer(pos as i64),
                    Err(_) => Object::Integer(-1),
                },
                _ => Object::Error("ftell arg must be file".to_string()),
            }
        }),
    );

    // fseek(file, offset, whence) (whence: 0=Start, 1=Current, 2=End)
    env_mut.set(
        "fseek".to_string(),
        Object::Builtin(|args| {
            if args.len() != 3 {
                return Object::Error("fseek expected 3 args".to_string());
            }
            match &args[0] {
                Object::File(handle) => {
                    let offset = match args[1] {
                        Object::Integer(i) => i,
                        _ => return Object::Error("fseek offset must be int".to_string()),
                    };
                    let whence = match args[2] {
                        Object::Integer(i) => i,
                        _ => return Object::Error("fseek whence must be int".to_string()),
                    };

                    let pos = match whence {
                        0 => SeekFrom::Start(offset as u64),
                        1 => SeekFrom::Current(offset),
                        2 => SeekFrom::End(offset),
                        _ => return Object::Error("invalid whence".to_string()),
                    };

                    let mut fh = handle.borrow_mut();
                    match fh.file.seek(pos) {
                        Ok(_) => {
                            fh.eof = false;
                            Object::Integer(0)
                        }
                        Err(_) => {
                            fh.error = true;
                            Object::Integer(-1)
                        }
                    }
                }
                _ => Object::Error("fseek arg must be file".to_string()),
            }
        }),
    );

    // rewind(file)
    env_mut.set(
        "rewind".to_string(),
        Object::Builtin(|args| {
            if args.len() != 1 {
                return Object::Error("rewind expected 1 arg".to_string());
            }
            match &args[0] {
                Object::File(handle) => {
                    let mut fh = handle.borrow_mut();
                    let _ = fh.file.seek(SeekFrom::Start(0));
                    fh.eof = false;
                    fh.error = false;
                    Object::Null
                }
                _ => Object::Error("rewind arg must be file".to_string()),
            }
        }),
    );

    // remove(path)
    env_mut.set(
        "remove".to_string(),
        Object::Builtin(|args| {
            if args.len() != 1 {
                return Object::Error("remove expected 1 arg".to_string());
            }
            let path = match &args[0] {
                Object::String(s) => s,
                _ => return Object::Error("remove arg must be string".to_string()),
            };
            if let Err(e) = std::fs::remove_file(path) {
                Object::Error(format!("remove failed: {}", e))
            } else {
                Object::Null
            }
        }),
    );

    // rename(old, new)
    env_mut.set(
        "rename".to_string(),
        Object::Builtin(|args| {
            if args.len() != 2 {
                return Object::Error("rename expected 2 args".to_string());
            }
            let old = match &args[0] {
                Object::String(s) => s,
                _ => return Object::Error("rename old must be string".to_string()),
            };
            let new = match &args[1] {
                Object::String(s) => s,
                _ => return Object::Error("rename new must be string".to_string()),
            };

            if let Err(e) = std::fs::rename(old, new) {
                Object::Error(format!("rename failed: {}", e))
            } else {
                Object::Null
            }
        }),
    );

    // getchar()
    env_mut.set(
        "getchar".to_string(),
        Object::Builtin(|args| {
            if !args.is_empty() {
                return Object::Error("getchar expected 0 args".to_string());
            }
            let mut buf = [0; 1];
            let mut handle = std::io::stdin();
            match handle.read(&mut buf) {
                Ok(0) => Object::Null, // EOF
                Ok(_) => Object::String((buf[0] as char).to_string()),
                Err(_) => Object::Error("getchar read error".to_string()),
            }
        }),
    );

    // Aliases

    // getc = fgetc (technically getc(stream), getchar() is stdin)
    // For now, I'll copy the logic of fgetc for getc.
    env_mut.set(
        "getc".to_string(),
        Object::Builtin(|args| {
            // Same logic as fgetc
            if args.len() != 1 {
                return Object::Error("getc expected 1 arg".to_string());
            }
            match &args[0] {
                Object::File(handle) => {
                    let mut fh = handle.borrow_mut();
                    let mut buf = [0; 1];
                    match fh.file.read(&mut buf) {
                        Ok(0) => {
                            fh.eof = true;
                            Object::Null
                        }
                        Ok(_) => Object::String((buf[0] as char).to_string()),
                        Err(_) => {
                            fh.error = true;
                            Object::Null
                        }
                    }
                }
                _ => Object::Error("getc arg must be file".to_string()),
            }
        }),
    );

    // putc = fputc
    env_mut.set(
        "putc".to_string(),
        Object::Builtin(|args| {
            if args.len() != 2 {
                return Object::Error("putc expected 2 arguments".to_string());
            }
            let c_str = args[0].inspect();
            let c = if let Some(ch) = c_str.chars().next() {
                ch
            } else {
                return Object::Null;
            };

            match &args[1] {
                Object::File(handle) => {
                    let mut fh = handle.borrow_mut();
                    if let Err(e) = write!(fh.file, "{}", c) {
                        fh.error = true;
                        Object::Error(format!("putc failed: {}", e))
                    } else {
                        Object::Null
                    }
                }
                _ => Object::Error("putc arg must be file".to_string()),
            }
        }),
    );

    // getc -> fgetc
    // putc -> fputc
    // rewind is already there. Wait, I added rewind.

    // We can just reuse the function pointers if we had them or just redefine.
    // simpler to just call the other builtin if I could look it up, but I can't.
    // Redefining is fine.
}
