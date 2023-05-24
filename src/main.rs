use std::env;
use std::process;
use std::path::Path;
use std::fs;
use futures::future::join_all;
use futures::executor::block_on;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() <= 1 {
        eprintln!("not have command!");
        process::exit(1);
    }

    let command: String = args[1].clone();
    let input_path = Path::new("./");
    let mut filter_compose: Option<&String> = None;

    if args.len() > 2 {
        // let path = Path::new(&args[2]);
        // if path.exists() {
        //     input_path = path;
        // } else {
        //     filter_compose = Some(&args[2]);
        // }
        filter_compose = Some(&args[2]);
    }
    println!("search {}", input_path.display());
    let compose_paths = search_compose_dir(&input_path);

    match command.as_str() {
        "list" => {
            match filter_compose {
                Some(f_compose) => {
                    let mut paths = Vec::new();
                    for compose in compose_paths {
                        if compose.contains(f_compose) {
                            paths.push(compose);
                        }
                    }
                    list_compose_dir(&paths);
                },
                _ => {
                    list_compose_dir(&compose_paths);
                },
            }
        },
        "start" => {
            match filter_compose {
                Some(f_compose) => {
                    let mut paths = Vec::new();
                    for compose in compose_paths {
                        let path = Path::new(&compose);
                        let filename = path.file_name().unwrap().to_str().unwrap();

                        if filename == *f_compose || filename.contains(f_compose) {
                            paths.push(compose);
                        }
                    }
                    println!("接下来将会启动这些项目: ");
                    list_compose_dir(&paths);
                    println!("(y/n):");
                    let mut input_string = String::new();
                    std::io::stdin().read_line(&mut input_string).unwrap();
                    if input_string.trim() == "y" {
                        println!("启动中......");
                        start_composes(&paths);
                    }
                },
                _ => {
                    start_composes(&compose_paths);
                },
            }
        },
        "stop" => {
            match filter_compose {
                Some(f_compose) => {
                    let mut paths = Vec::new();
                    for compose in compose_paths {
                        let path = Path::new(&compose);
                        let filename = path.file_name().unwrap().to_str().unwrap();

                        if filename == *f_compose || filename.contains(f_compose) {
                            paths.push(compose);
                        }
                    }
                    println!("接下来将会关闭这些项目: ");
                    list_compose_dir(&paths);
                    println!("(y/n):");
                    let mut input_string = String::new();
                    std::io::stdin().read_line(&mut input_string).unwrap();
                    if input_string.trim() == "y" {
                        println!("关闭中......");
                        stop_composes(&paths);
                    }
                },
                _ => {
                    stop_composes(&compose_paths);
                },
            }
        },
        _ => {
            println!("not have command!");
        },
    }
}

fn is_compose_dir(input_path: &Path) -> bool {
    if !input_path.is_dir() {
        return false;
    }

    // println!("> {:?}", input_path);
    // 判断这个文件夹是否存在docker-compose.yaml or yml 文件
    let mut is_compose = false;
    match fs::read_dir(input_path) {
        Err(why) => {
            println!("! {:?}", why.kind());
            is_compose = false;
        },
        Ok(dir_paths) => {
            for path in dir_paths {
                let real_path = path.unwrap().path();
                let filename = real_path.file_name().unwrap().to_str().unwrap();
                let compose_filenames = ["docker-compose.yaml", "docker-compose.yml"];
                is_compose = real_path.is_file() && compose_filenames.contains(&filename);
                if is_compose {
                    break;
                }
            }
        }
    }
    is_compose
}

fn search_compose_dir(input_path: &Path) -> Vec<String> {
    let mut compose_paths: Vec<String> = Vec::new();
    match fs::read_dir(input_path) {
        Err(why) => println!("! {:?}", why.kind()),
        Ok(paths) => {
            for path in paths {
                let real_path = path.unwrap().path();
                let is_compose = is_compose_dir(&real_path);
                if is_compose {
                    // println!("find compose dir: {}", real_path.display());
                    compose_paths.push(real_path.to_str().unwrap().to_string());
                }
            }
        }
    }
    compose_paths
}

fn print_n(print_str: &String, n: i32) {
    for _ in 0..n {
        print!("{}", print_str);
    }
    println!("");
}

enum ComposeStatus {
    Start,
    Stop
}

async fn get_compose_status(path_str: &String) -> Result<(String, ComposeStatus), String> {
    let command = format!("cd '{}' && sudo docker compose top", path_str);
    match process::Command::new("bash").arg("-c").arg(command).output() {
        Ok(output) =>  {
            let out = String::from_utf8(output.stdout).unwrap();
            let path = Path::new(path_str);
            let filename = path.file_name().unwrap().to_str().unwrap().to_string();

            if out == "" {
                Ok((filename, ComposeStatus::Stop))
            } else {
                Ok((filename, ComposeStatus::Start))
            }
        },
        Err(error) => Err(error.to_string())
    }
}

fn list_compose_dir(compose_paths: &Vec<String>) {
    let mut compose_task = Vec::new();

    for compose in compose_paths {
        let task = get_compose_status(&compose);
        compose_task.push(task);
    }

    println!("状态检测中......");
    let results = block_on(join_all(compose_task));

    // Print result table
    let f_n = 57;
    let print_driver = String::from("=");
    print_n(&print_driver, f_n);
    println!("{:50} {:20}", "Project", "Status");
    print_n(&print_driver, f_n);

    for result in results {
        match result {
            Ok((filename, status)) => {
                match status {
                    ComposeStatus::Stop => {
                        println!("{:50} {:20}", filename, "down");
                    },
                    ComposeStatus::Start => {
                        println!("{:50} {:20}", filename, "start");
                    }
                }
            },
            Err(error) => {
                eprintln!("{}", error);
            },
        }
    }

    print_n(&print_driver, f_n);
}

async fn start_compose(path_str: &String) -> Result<String, String> {
    let command = format!("cd '{}' && sudo docker compose up -d", path_str);
    match process::Command::new("bash").arg("-c").arg(command).output() {
        Ok(output) => Ok(String::from_utf8(output.stdout).unwrap()),
        Err(error) => Err(error.to_string())
    }
}

fn start_composes(compose_paths: &Vec<String>) {
    let mut compose_task = Vec::new();
    for compose in compose_paths {
        // let path = Path::new(compose);
        let task = start_compose(&compose);
        compose_task.push(task);
    }

    let results = block_on(join_all(compose_task));

    for result in results {
        match result {
            Err(error) => eprintln!("{}", error),
            _ => (),
        }
    }

    println!("所有任务完成!")
}

async fn stop_compose(path_str: &String) -> Result<String, String> {
    let command = format!("cd '{}' && sudo docker compose down", path_str);
    match process::Command::new("bash").arg("-c").arg(command).output() {
        Ok(output) => Ok(String::from_utf8(output.stdout).unwrap()),
        Err(error) => Err(error.to_string())
    }
}

fn stop_composes(compose_paths: &Vec<String>) {
    let mut compose_task = Vec::new();
    for compose in compose_paths {
        let task = stop_compose(&compose);
        compose_task.push(task);
    }

    let results = block_on(join_all(compose_task));

    for result in results {
        match result {
            Err(error) => eprintln!("{}", error),
            _ => (),
        }
    }

    println!("所有任务完成!")
}
