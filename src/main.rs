use std::env;
use std::process;
use std::path::Path;
use std::fs;
use futures::future::join_all;
use futures::executor::block_on;

fn main() {
    println!("Hello, world!");
    let args: Vec<String> = env::args().collect();
    if args.len() < 1 {
        eprintln!("not have command!");
        process::exit(1);
    }

    let command: String = args[1].clone();
    let mut input_path = Path::new("./");

    if args.len() > 2 {
        input_path = Path::new(&args[2]);
    }
    println!("list {}", input_path.display());
    let compose_paths = search_compose_dir(&input_path);

    match command.as_str() {
        "list" => {
            list_compose_dir(&compose_paths);
        },
        "start" => {
            start_composes(&compose_paths);
            println!("start");
        },
        "stop" => {
            println!("stop");
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
                    println!("find compose dir: {}", real_path.display());
                    compose_paths.push(real_path.to_str().unwrap().to_string());
                }
            }
        }
    }
    compose_paths
}

fn print_n(print_str: String, n: i32) {
    for _ in 0..n {
        print!("{}", print_str);
    }
    println!("");
}

fn list_compose_dir(compose_paths: &Vec<String>) {
    let f_n = 57;
    print_n(String::from("-"), f_n);
    println!("{:50} {:20}", "Project", "Status");
    print_n(String::from("-"), f_n);
    for compose in compose_paths {
        let path = Path::new(compose);
        let filename = path.file_name().unwrap().to_str().unwrap();

        let command = format!("cd '{}' && sudo docker compose top", compose);
        let output = process::Command::new("bash").arg("-c").arg(command).output().expect("命令运行异常");
        let out = String::from_utf8(output.stdout).unwrap();

        if out == "" {
            println!("{:50} {:20}", filename, "down");
        } else {
            println!("{:50} {:20}", filename, "start");
        }
    }
    print_n(String::from("-"), f_n);
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
