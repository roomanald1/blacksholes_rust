use std::io;
use std::io::Write;
use std::os::fd::AsRawFd;
use crate::stream::start_streaming;

pub mod blacksholes;
mod lib;
mod stream;
mod alloc_tracker;



#[tokio::main]
async fn main()  {
    disable_stdout_buffering().unwrap();
    let _dhat = dhat::Profiler::new_heap();

    let mut ctrl_c = tokio::signal::ctrl_c();
    tokio::select! {
        _ = start_streaming() => {
            //println!("Streaming completed");
        },
        _ =  ctrl_c => {
            //println!("Ctrl+C");
        },
    }
    //Comment out above and just run below to get dhat memory output
    //Also need to uncomment line from alloc_tracker
    //profile_blacksholes()
}

fn disable_stdout_buffering() -> io::Result<()> {
    // Flush any existing output first
    io::stdout().flush()?;

    unsafe {
        // Get stdout file descriptor
        let stdout_fd = io::stdout().as_raw_fd();

        // Open as FILE* for setvbuf
        let stdout = libc::fdopen(stdout_fd, b"w\0".as_ptr() as *const libc::c_char);
        if stdout.is_null() {
            return Err(io::Error::last_os_error());
        }

        // Disable all buffering
        if libc::setvbuf(stdout, std::ptr::null_mut(), libc::_IONBF, 0) != 0 {
            return Err(io::Error::last_os_error());
        }
    }
    Ok(())
}



fn profile_blacksholes(){
    stream::calculate_options(100.0, 1.0, 0.05, 98.0, 0.2);
}


async fn calculate_options(){
    stream::calculate_options(100.0, 1.0, 0.05, 98.0, 0.2);
}

