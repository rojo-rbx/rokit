use winapi::um::processthreadsapi::GetCurrentProcessId;
use winapi::um::wincon::GetConsoleProcessList;

use super::Launcher;

pub async fn try_detect_launcher() -> Option<super::Launcher> {
    tracing::debug!("trying to detect launcher using Windows API");

    /*
        Allocate a buffer for process IDs - we need space for at
        least one ID, which will hopefully be our own process ID
    */
    let mut process_list = [0u32; 1];
    let process_id = unsafe { GetCurrentProcessId() };
    let process_count = unsafe { GetConsoleProcessList(process_list.as_mut_ptr(), 1) };

    tracing::debug!(
        id = %process_id,
        count = %process_count,
        "got process id and count"
    );

    /*
        If there's only one process (our process), the console will be destroyed on exit,
        this very likely means it was launched from Explorer or a similar environment.

        A similar environment could be the download folder in a web browser,
        launching the program directly using the "Run" dialog, ..., but for
        simplicity we'll just assume it was launched from Explorer.
    */
    if process_count == 1 && process_list[0] == process_id {
        Some(Launcher::WindowsExplorer)
    } else {
        None
    }
}
