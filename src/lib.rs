use core::fmt::{Display, Formatter};
use futures::executor::block_on;
use futures::future::join_all;
use std::fs;
use std::process;
use std::path::Path;

mod utils;

use utils::{ba_error, is_yes, print_n, BDEResult};

#[derive(PartialEq, Eq)]
pub enum ComposeStatus {
    Start,
    Stop,
}

pub enum ComposeCommand {
    Start,
    Stop,
    Restart,
    Unknown,
}

impl Display for ComposeStatus {
    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
        match self {
            ComposeStatus::Start => write!(f, "Start"),
            ComposeStatus::Stop => write!(f, "Stop"),
        }
    }
}

impl From<ComposeCommand> for String {
    fn from(compose_command: ComposeCommand) -> Self {
        match compose_command {
            ComposeCommand::Start => "启动".into(),
            ComposeCommand::Stop => "关闭".into(),
            ComposeCommand::Restart => "重启".into(),
            ComposeCommand::Unknown => "未知命令".into(),
        }
    }
}

impl From<String> for ComposeCommand {
    fn from(command: String) -> Self {
        match command.as_str() {
            "start" => ComposeCommand::Start,
            "stop" => ComposeCommand::Stop,
            "restart" => ComposeCommand::Restart,
            _ => ComposeCommand::Unknown,
        }
    }
}

pub struct DockerCompose {
    pub docker_name: String,
    path: String,
    pub status: ComposeStatus,
}

impl DockerCompose {
    pub fn build(path: &Path) -> Option<DockerCompose> {
        let composep = is_compose_dir(path);

        let docker_name = path.file_name().unwrap().to_str().unwrap().to_string();
        let path_str = path.display().to_string();

        if composep {
            Some(DockerCompose {
                docker_name,
                path: path_str,
                status: ComposeStatus::Stop,
            })
        } else {
            None
        }
    }

    pub async fn refresh_status(&mut self) -> BDEResult<()> {
        let command = format!("cd '{}' && sudo docker compose top", &self.path);
        match process::Command::new("bash")
            .arg("-c")
            .arg(command)
            .output()
        {
            Ok(output) => {
                let out = String::from_utf8(output.stdout).unwrap();

                if out == "" {
                    self.status = ComposeStatus::Stop;
                } else {
                    self.status = ComposeStatus::Start;
                }

                Ok(())
            }
            Err(error) => Err(ba_error(
                format!("执行命令失败: {}", error.to_string()).as_mut_str(),
            )),
        }
    }

    pub async fn run_command(&self, command: &ComposeCommand) -> BDEResult<String> {
        let command = match command {
            ComposeCommand::Start => "up -d",
            ComposeCommand::Stop => "down",
            ComposeCommand::Restart => "restart",
            ComposeCommand::Unknown => return Err(ba_error("未知命令")),
        };
        let command = format!("cd '{}' && sudo docker compose {}", self.path, command);
        match process::Command::new("bash")
            .arg("-c")
            .arg(command)
            .output()
        {
            Ok(output) => Ok(String::from_utf8(output.stdout).unwrap()),
            Err(error) => Err(ba_error(
                format!("执行命令失败: {}", error.to_string()).as_mut_str(),
            )),
        }
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
        }
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

pub fn search_compose_dir(
    input_path: &Path,
    filter: Option<String>,
) -> BDEResult<Vec<DockerCompose>> {
    let mut composes: Vec<DockerCompose> = Vec::new();

    for entry in fs::read_dir(input_path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            if let Some(docker_compose) = DockerCompose::build(&path) {
                if let Some(filter_name) = filter.clone() {
                    let filename = path.file_name().unwrap().to_str().unwrap();
                    if filename.contains(&filter_name) || filename == filter_name {
                        composes.push(docker_compose);
                    }
                } else {
                    composes.push(docker_compose);
                }
            }
        }
    }

    Ok(composes)
}

pub fn refresh_composes_status(composes: &mut Vec<DockerCompose>) -> BDEResult<()> {
    if composes.len() < 1 {
        return Err(ba_error("没有找到任何项目"));
    }

    let mut compose_task = Vec::new();

    for compose in composes.iter_mut() {
        let task = compose.refresh_status();
        compose_task.push(task);
    }

    let results = block_on(join_all(compose_task));

    for result in results {
        match result {
            Err(error) => {
                eprintln!("{}", error);
            }
            _ => {}
        }
    }

    Ok(())
}

pub fn run_composes_command(
    composes: &Vec<&DockerCompose>,
    command: &ComposeCommand,
) -> BDEResult<()> {
    if composes.len() < 1 {
        return Err(ba_error("没有找到任何项目"));
    }

    let mut compose_task = Vec::new();

    for compose in composes {
        let task = compose.run_command(&command);
        compose_task.push(task);
    }

    let results = block_on(join_all(compose_task));

    for result in results {
        match result {
            Ok(output) => {
                println!("{}", output);
            }
            Err(error) => {
                eprintln!("{}", error);
            }
        }
    }

    Ok(())
}

fn print_docker_compose_status(docker_composes: &Vec<DockerCompose>) {
    // Print result table
    let f_n = 57;
    let print_driver = String::from("=");
    print_n(&print_driver, f_n);
    println!("{:50} {:20}", "Project", "Status");
    print_n(&print_driver, f_n);

    for compose in docker_composes {
        println!("{:50} {:20}", compose.docker_name, compose.status);
    }

    print_n(&print_driver, f_n);
}

pub fn run(command: String, path: &Path, filter_name: Option<String>) -> BDEResult<()> {
    let mut docker_composes = search_compose_dir(path, filter_name)?;

    let compose_command = ComposeCommand::from(command.clone());

    println!("状态检测中......");
    refresh_composes_status(&mut docker_composes)?;

    match compose_command {
        ComposeCommand::Start => {
            println!("接下来会启动以下项目:");
            let mut start_composes: Vec<&DockerCompose> = Vec::new();
            for compose in docker_composes.iter() {
                if compose.status == ComposeStatus::Stop {
                    start_composes.push(compose);
                }
            }
            let executorp = is_yes("是否执行");
            if executorp {
                println!("启动中......");
                run_composes_command(&start_composes, &compose_command)?;
                refresh_composes_status(&mut docker_composes)?;
                print_docker_compose_status(&docker_composes);
            }
        }
        ComposeCommand::Stop => {
            println!("接下来会关闭以下项目:");
            let mut stop_composes: Vec<&DockerCompose> = Vec::new();
            for compose in docker_composes.iter() {
                if compose.status == ComposeStatus::Start {
                    stop_composes.push(compose);
                }
            }
            let executorp = is_yes("是否执行");
            if executorp {
                println!("关闭中......");
                run_composes_command(&stop_composes, &compose_command)?;
                refresh_composes_status(&mut docker_composes)?;
                print_docker_compose_status(&docker_composes);
            }
        }
        ComposeCommand::Restart => {
            println!("接下来会重启以下项目:");
            let mut restart_composes: Vec<&DockerCompose> = Vec::new();
            for compose in docker_composes.iter() {
                if compose.status == ComposeStatus::Start {
                    restart_composes.push(compose);
                }
            }
            let executorp = is_yes("是否执行");
            if executorp {
                println!("重启中......");
                run_composes_command(&restart_composes, &compose_command)?;
                refresh_composes_status(&mut docker_composes)?;
                print_docker_compose_status(&docker_composes);
            }
        }
        ComposeCommand::Unknown => {
            if command.as_str() == "status" {
                print_docker_compose_status(&docker_composes);
            } else {
                return Err(ba_error("未知命令"));
            }
        }
    }

    Ok(())
}
