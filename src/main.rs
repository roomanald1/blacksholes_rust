
use crate::stream::start_streaming;

pub mod blacksholes;
mod lib;
mod stream;
mod alloc_tracker;



#[tokio::main]
async fn main()  {
    tokio::select! {
        _ = start_streaming() => {
            println!("Streaming completed");
        },
        _ = tokio::signal::ctrl_c() => {
            println!("Ctrl+C");
        },
    }
    //Comment out above and just run below to get dhat memory output
    //Also need to uncomment line from alloc_tracker
    //profile_blacksholes()
}



fn profile_blacksholes(){
    let _dhat = dhat::Profiler::new_heap();
    stream::calculate_options(100.0, 1.0, 0.05, 98.0, 0.2);
}


async fn calculate_options(){
    stream::calculate_options(100.0, 1.0, 0.05, 98.0, 0.2);
}

