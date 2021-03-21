use std::io::{stdin, stdout, Write};
use std::path::Path;
use std::env;
use libc;
use std::fs;

fn main() {
    println!("*------Author: Zhang YB ID: 18281253@bjtu.edu.cn-------*");
    loop {
        print!("{} =>> ", env::current_dir().unwrap().to_str().unwrap());  // prompt
        stdout().flush().expect("Error: can not flush the output stream."); // 刷新标准输出

        let mut input = String::new();
        stdin().read_line(&mut input).expect("Error: can not read your input.");
        // 处理分割输入命令与参数
        let mut parts = input.trim().split_whitespace();
        let command = parts.next().unwrap();
        let mut args = parts;
                
        match command {
            //构建内部指令
            "cd" => { // 内部命令1 改变目录
                let new_dir = args.peekable().peek().map_or("/", |x| *x);
                let root = Path::new(new_dir);
                if let Err(e) = env::set_current_dir(root) {
                    eprintln!("{}", e);
                }
            },
            "touch" => { //内部命令2 创建文本文件
                let docs: Vec<String> = args.peekable().map(|s| s.to_owned() + "\0").collect();
                if docs.len() < 1 {
                    eprintln!("You need input the file name.");
                    continue;
                }
                let create = |x: &String| unsafe {
                    if let -1 = libc::open(x.as_bytes().as_ptr() as *const i8,
                        libc::O_CREAT | libc::O_TRUNC | libc::O_RDWR,
                        libc::S_IRUSR | libc::S_IWUSR | libc::S_IRGRP | libc::S_IROTH) {
                        eprintln!("Can not create the file '{}'", x);
                    }
                };
                docs.iter().for_each(create);
            },
            "cat" => { // 内部命令3 显示文件内容
                args.map(|x| fs::read_to_string(x))
                    .for_each(|x| match x {
                        Ok(str) => println!("{}", str),
                        Err(e) => eprintln!("\nError: {}",e),
                    });
            },
            "rm" => { // 内部命令4 删除文件&目录
                let mut names: Vec<String> = args.peekable().map(|x| x.to_owned()).collect();
                let mut is_dir = false;

                if names.len() < 1 {
                    eprintln!("You need input the file name.");
                    continue;
                } else {
                    match names[0].as_str() {
                        "-rf" | "-fr" => {
                            if names.len() < 2 {
                                eprintln!("You need input the dir name.");
                                continue;
                            }
                            is_dir = true;
                            names.remove(0);
                        },
                        _ => (),
                    }
                }
                names.iter().for_each(|x| delete_name(&x, is_dir));
            },
            "mkdir" => { //内部命令5 创建目录
                let dirs: Vec<String> = args.peekable().map(|s| s.to_owned()).collect();
                if dirs.len() < 1 {
                    eprintln!("You need input the dir name.");
                    continue;
                }
                dirs.iter().for_each(|x| {
                    match fs::metadata(x) {
                        Ok(meta) if meta.is_dir() => eprintln!("'{}' is allready exist dir", x),
                        Ok(_) => eprintln!("'{}' is allready exist file", x),
                        Err(_) => if let -1 = unsafe { 
                            libc::mkdir((x.to_owned()+"\0").as_bytes().as_ptr() as *const i8, 0o777) } {
                                eprintln!("Error: Unable to create dir {}",x);
                        },
                    }
                })
            },
            "cp" => { // 内部命令6 拷贝内容
                let src = match args.next() {
                    Some(s) => s,
                    None => { eprintln!("You need input the source file."); continue; },
                };
                let dst = match args.next() {
                    Some(d) => d,
                    None => { eprintln!("You need input the destination file."); continue; },
                };
                let mut char_buf: [i8; 2048] = [0; 2048];
                unsafe { // 拷贝部分
                    let src_file = libc::syscall(libc::SYS_open, (src.to_owned()+"\0").as_ptr() as *const i8, libc::O_RDONLY);
                    if src_file < 0 { eprintln!("can not open file `{}`", src); continue; }
                    let dst_file = libc::syscall(libc::SYS_creat, (dst.to_owned()+"\0").as_ptr() as *const i8, 0o666);
                    if dst_file < 0 {eprintln!("can not create file `{}`", dst); continue; }

                    loop {
                        match libc::syscall(libc::SYS_read, src_file, char_buf.as_mut_ptr() as *mut i8, char_buf.len()) {
                            0 => break,
                            count => libc::syscall(libc::SYS_write, dst_file, char_buf.as_ptr() as * const i8, count),
                        };
                    }
                    libc::syscall(libc::SYS_close, dst_file);
                    libc::syscall(libc::SYS_close, src_file);
                }
            },
            "export" => { // 内部命令7 添加path
                let paths: Vec<String> = args.peekable().map(|s| s.to_owned()+"\n").collect();
                if paths.len() < 1 { eprintln!("You need at least input one path."); continue; }
                let config_name = b".srshrc\0";
                unsafe {
                    let mut rc_file = libc::syscall(libc::SYS_open, config_name.as_ptr() as *const i8, libc::O_APPEND | libc::O_WRONLY);
                    if rc_file < 0 {
                        rc_file = libc::syscall(libc::SYS_creat, config_name.as_ptr() as *const i8, 0o666);
                    }
                    paths.iter().for_each(|s| {
                        if libc::syscall(libc::SYS_write, rc_file, s.as_ptr() as *const i8, s.len()) <= 0 {
                            eprint!("can not write {}", s);
                        }
                    });
                    libc::syscall(libc::SYS_close, rc_file);
                }
            },
            "help" => println!("cd\t<dir>\t\t改变当前目录\n\
                                cp\t<src>\t<dst>\t拷贝文件\n\
                                rm\t<file>\t\t删除文件\n\
                                \t-rf <dir>\t递归删除目录\n\
                                cat\t<file>\t\t打印文本文件内容至终端\n\
                                help\t\t\t显示帮助\n\
                                touch\t<file>\t\t新建文本文件\n\
                                mkdir\t<dirname>\t创建新目录\n\
                                export\t<path>\t\t添加path\n\
                                exit\t\t\t退出shell\n"), // 内部命令8 帮助
            "exit" => return, // 内部命令9 退出shell程序
            command => { // 外部命令
                let args = input.trim().split_whitespace();
                let mut args: Vec<String> = args.peekable().map(|s| s.to_owned() + "\0").collect();
                args[0] = find_command(command); // 寻找path下之否有该命令
                let mut argv: Vec<* const i8> = args.iter().map(|s| s.as_bytes().as_ptr() as *const i8).collect();
                argv.push(std::ptr::null());
                unsafe {
                    let pid = libc::fork();
                    if pid < 0 {
                        eprintln!("Failed to create new process");
                    } else if pid == 0 {
                        if let -1 = libc::execv(*argv.get(0).unwrap(), argv.as_ptr()) { // 这里没有用execvp，命令路径计算得出
                            eprintln!("Error: Can not find the command of program {}!", command);
                            return; // 子进程返回
                        }
                    } else {
                            libc::wait(0 as *mut i32);
                    }
                }
            },
        }
    }
}

fn find_command(command: &str) -> String {
    let paths = match fs::read_to_string(".srshrc") {
        Ok(s) => s,
        Err(_) => return command.to_owned() + "\0",
    };
    for path in paths.split_ascii_whitespace() {
        let path = Path::new(path).join(command);
        match fs::metadata(&path) {
            Ok(meta) if meta.is_file() => return path.to_owned().to_str().unwrap().to_owned() + "\0",
            Ok(_) | Err(_) => continue,
        }
    }
    return command.to_owned() + "\0";
}

fn delete_name(name: &str, is_dir: bool) { // 递归删除文件与目录
    match is_dir {
        false => {
            if let -1 = unsafe { libc::unlink((name.to_owned()+"\0").as_bytes().as_ptr() as *const i8) } {
                eprintln!("Error: can not delete the file '{}'", name);
            }; 
        },
        true => {
            let iter = fs::read_dir(Path::new(name)).expect("Can not read the directory.");
            iter.filter(|x| x.is_ok())
                .map(|x| x.unwrap())
                .for_each(|x| delete_name(x.path().to_str().unwrap(), x.metadata().unwrap().is_dir()));
            if let -1 = unsafe { libc::rmdir((name.to_owned()+"\0").as_bytes().as_ptr() as *const i8)} {
                eprintln!("Error: can not remove the dir '{}'", name);
            }
        },
    }
}