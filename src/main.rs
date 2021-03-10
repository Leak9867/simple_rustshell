use std::io::{stdin, stdout, Write};
use std::path::Path;
use std::env;
use libc;
use std::fs;

fn main() {
    loop {
        print!("{} =>> ", env::current_dir().unwrap().to_str().unwrap());  // prompt
        stdout().flush().expect("Error: can not flush the output stream."); // 刷新标准输出

        let mut input = String::new();
        stdin().read_line(&mut input).expect("Error: can not read your input.");
        // 处理分割输入命令与参数
        let mut parts = input.trim().split_whitespace();
        let command = parts.next().unwrap();
        let args = parts;
                
        match command {
            //构建内部指令
            "cd" => { // 内部命令1
                let new_dir = args.peekable().peek().map_or("/", |x| *x);
                let root = Path::new(new_dir);
                if let Err(e) = env::set_current_dir(root) {
                    eprintln!("{}", e);
                }
            },
            "touch" => { //内部命令2
                let docs: Vec<String> = args.peekable().map(|s| s.to_owned() + "\0").collect();
                if docs.len() < 1 {
                    eprintln!("You need input the file name.");
                    continue;
                }
                for doc in docs {
                    unsafe { 
                        if let -1 = libc::open(doc.as_bytes().as_ptr() as *const i8,
                                libc::O_CREAT | libc::O_TRUNC | libc::O_RDWR,
                                libc::S_IRUSR | libc::S_IWUSR | libc::S_IRGRP | libc::S_IROTH) {
                                    eprintln!("Can not create the file '{}'", doc);
                        }
                    }
                }
            },
            "cat" => { // 内部命令3
                let mut txt = "".to_owned();
                for arg in args {
                    match fs::read_to_string(arg) {
                        Ok(str) => txt = str,
                        Err(e) => eprintln!("Error: {}", e),
                    }
                    println!("{}", txt);
                }
            },
            "rm" => { // 内部命令4 
                let files: Vec<String> = args.peekable().map(|s| s.to_owned() + "\0").collect();
                if files.len() < 1 {
                    eprintln!("You need input the file name.");
                    continue;
                }
                for file in files {
                    if let -1 = unsafe { libc::unlink(file.as_bytes().as_ptr() as *const i8) } {
                        eprintln!("Error: can not delete the file '{}'", file);
                    }
                }
            },
            "mkdir" => { //内部命令5
                let chk_dirs: Vec<String> = args.peekable().map(|s| s.to_owned()).collect();
                if chk_dirs.len() < 1 {
                    eprintln!("You need input the dir name.");
                    continue;
                }
                let dirs: Vec<String> = chk_dirs.iter().map(|s| s.clone() + "\0").collect();
                for (i, chk_dir) in chk_dirs.iter().enumerate() {
                    if let Ok(meta) = fs::metadata(&chk_dir) {
                        if meta.is_dir() {
                            eprintln!("'{}' is allready exist dir", chk_dir);
                        } else {
                            eprintln!("'{}' is allready exist file", chk_dir);
                        }
                        continue;
                    }
                    if let -1 = unsafe { libc::mkdir(dirs[i].as_bytes().as_ptr() as *const i8, 0o777) } {
                        eprintln!("Unable to create dir {}", chk_dir);
                    }
                }
            },
            "exit" => return, // 内部命令7 退出shell程序
            command => { // 外部命令
                let args = input.trim().split_whitespace();
                let args: Vec<String> = args.peekable().map(|s| s.to_owned() + "\0").collect();
                let mut argv: Vec<* const i8> = args.iter().map(|s| s.as_bytes().as_ptr() as *const i8).collect();
                argv.push(std::ptr::null());
                
                unsafe {
                    let pid = libc::fork();
                    if pid < 0 {
                        eprintln!("Failed to create new process");
                    } else if pid == 0 {
                        if let -1 = libc::execvp(*argv.get(0).unwrap(), argv.as_ptr()) {
                            eprintln!("Error: Can not find the command of program {}!", command);
                            return; // 子进程返回
                        }
                    } else {
                        let _ = libc::wait(0 as *mut i32);
                    }
                }
            },
        }
    }
}
