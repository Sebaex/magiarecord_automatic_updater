extern crate reqwest;
extern crate zip;

use std::collections::HashMap;
use std::io;
use std::fs;
use std::fs::File;
use std::process::Command;
use std::env;
use std::path::Path;



fn main() {
    let args: Vec<String> = env::args().collect();
    let force_local_adb_flag = args.iter().any(|x| x == "-forceLocalADB");
    let force_adb_download_flag = args.iter().any(|x| x == "-forceADBDownload");
    let no_install_flag = args.iter().any(|x| x == "-noInstall");
    let do_nothing_flag = args.iter().any(|x| x == "-doNothing");
    let device_type_option;
    let emulator_type_option;
    let emulators_ports = HashMap::from([
        ("nox", "62001"),
        ("wsa", "58526"),
    ]);

    let mut switch_option : String = String::new();
    let mut device_type : String = String::new();
    let mut emulator_type : String = String::new();
    //let mut apk_type = String::new();

    if do_nothing_flag {
        return;
    }

    if force_adb_download_flag || ((!detect_native_adb() || force_local_adb_flag)  && !detect_prev_downloaded_adb()) {
        //print!("Downloading ADB");
        //todo, request confirmation of acceptance of adb licence
        get_platform_tools();
    }

    clear_console();
    print!("Friendly reminder to search how to turn on usb debugging in your device");
    println!("Select your device\n 1.- Phone/Bluestacks\n 2.- Other Emulator"); //Bluestacks doesn't require IP connection via adb to install an apk
    io::stdin().read_line(&mut switch_option).expect("A problem has occured");
    device_type_option = switch_option.trim().parse().expect("a number");
    match device_type_option{
        1 => device_type = String::from("phone"),
        2 => device_type = String::from("emulator"),
        _ => main()
    }

    clear_console();

    if device_type == "emulator" {
        switch_option = String::new();
        println!("Select your emulator\n 1.- Nox Player\n 2.- Windows Subsystem for Android");
        io::stdin().read_line(&mut switch_option).expect("A problem has occured");
        emulator_type_option = switch_option.trim().parse().expect("a number");
        match emulator_type_option{
            1 => emulator_type = String::from("nox"),
            2 => emulator_type = String::from("wsa"),
            _ => main()
        }
        connect_to_emu(force_local_adb_flag || !detect_native_adb(), emulators_ports.get(emulator_type.as_str()).unwrap(), emulator_type)
    }
    
    /* Pending if rayshift starts uploading releases in github, should check how to get last release whenever it's available
    println!("Device type: {}", device_type);
    println!("Select desired APK\n 1.- Vanilla APK\n 2.- Rayshift APK")
    */
    
    println!("Using JP APK\n");
    get_jp_apk();

    let phonepath = "/data/local/tmp/magiarecord.apk";

    if !no_install_flag {
        install_apk(force_local_adb_flag || !detect_native_adb(), phonepath);
    }
    
    println!("Done! -- Press Enter to continue");
    io::stdin().read_line(&mut String::new()).unwrap();
     
}

fn pause(){
    println!("Press enter to continue...");
    
    let mut pause = String::new();
    io::stdin().read_line(&mut pause).expect("Error... somehow");
}

fn clear_console() -> Result<(), io::Error> {
    if cfg!(target_os="windows"){
        Command::new("cmd")
            .args(&["/C", "cls"])
            .status()?;
    }
    else {
        Command::new("clear")
            .status()?;
    };
    Ok(())
}

fn connect_to_emu(local: bool, port: &&str, emulator_type: String){
    clear_console();
    let adbcmd = if !local {
        "adb"
    } else if cfg!(windows){
        ".\\platform-tools\\adb.exe"
    } else {
        "./platform-tools/adb"
    };

    if emulator_type == "wsa" {
        println!("ADB might say 'failed to authenticate', don't worry about and accept the usb debugging prompt that will pop up, if not, skip this message\nAnother note, since WSA doesn't have play services, you won't be able to whale");
        pause()
    }

    println!("Waiting for emulator connection in ADB\n");
    let mut adb_connect_emu = Command::new(adbcmd)
        .args(&["connect", format!("localhost:{}", port).as_str()])
        .spawn()
        .expect("Failed to wait for a device on adb");
    adb_connect_emu.wait().expect("Failed while connecting to emulator");
}

fn detect_prev_downloaded_adb() -> bool {
    let path_to_check = if cfg!(windows){
        ".\\platform-tools\\adb.exe"
    } else {
        "./platform-tools/adb"
    };
    return Path::new(path_to_check).exists();
}

fn detect_native_adb() -> bool {
    let res = which::which("adb").is_ok();
    return res;
}

fn get_url_for_platform_tools() -> String{
    return if cfg!(target_os="windows"){ //test windows
        String::from("https://dl.google.com/android/repository/platform-tools-latest-windows.zip")
    } else if cfg!(target_os="macos"){ //test mac
        String::from("https://dl.google.com/android/repository/platform-tools-latest-darwin.zip")
    } else if cfg!(target_os="linux"){ //test linux
        String::from("https://dl.google.com/android/repository/platform-tools-latest-linux.zip")
    } else {
        String::from("unknown")
    };
}

fn get_platform_tools(){
    //by using this you're accepting the licence agreement for the android platform tools:
    //see agreement here: https://developer.android.com/studio/releases/platform-tools
    //this entire block is hacky i should really use a native adb server emulation instead
    //but that does not exist, so instead this will have to do
    let target = get_url_for_platform_tools();
    if target == "unknown" {
        print!("Not a known os, you should manually configure ADB for your platform before running this script");
        std::process::exit(1);
    }
    let mut resp = reqwest::get(target.as_str())
        .expect("request failed");
    
    let mut out = File::create("platformtools.zip")
        .expect("failed to create file (platformtools)");
    
    let _fileoutresult = io::copy(&mut resp, &mut out)
        .expect("Failed to ADB from remote to local file");
    //release the filehandle
    drop(out);

    unzip_archive("platformtools.zip");
    
}

//citation: https://github.com/mvdnes/zip-rs/blob/master/examples/extract.rs
//under MIT Licence
fn unzip_archive(path: &str){ 
    let file = fs::File::open(path).unwrap();
    let mut archive = zip::ZipArchive::new(file).unwrap();
    
    for i in 0 .. archive.len() {
        let mut file_to_extract = archive.by_index(i).unwrap();
        let outpath = file_to_extract.mangled_name();

        if file_to_extract.name().ends_with('/') {
            fs::create_dir_all(&outpath).unwrap();
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(&p).unwrap();
                }
            }
            let mut outfile = fs::File::create(&outpath).unwrap();
            let _ = io::copy(&mut file_to_extract, &mut outfile);
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Some(mode) = file_to_extract.unix_mode() {
                fs::set_permissions(&outpath, fs::Permissions::from_mode(mode)).unwrap();
            }
        }
    }
    
}

fn get_apk(target: &str){
    //https://users.rust-lang.org/t/download-file-from-web/19863 3 lines and it worked lol
    let mut resp = reqwest::get(target).expect("request failed");
    let mut out = File::create("magiarecord.apk").expect("failed to create file");
    io::copy(&mut resp, &mut out).expect("failed to copy content");
}

fn get_jp_apk(){
    let target = "https://jp.rika.ren/apk/Origin/com.aniplex.magireco.arm8.apk";
    get_apk(target);
}

//TODO better fail state detection
fn install_apk(local: bool, phonepath: &str){
    let adbcmd = if !local {
        "adb"
    } else if cfg!(windows){
        ".\\platform-tools\\adb.exe"
    } else {
        "./platform-tools/adb"
    };

    println!("Waiting for Android device on ADB\n");
    let mut adb_wait_process = Command::new(adbcmd)
        .arg("wait-for-device")
        .spawn()
        .expect("Failed to wait for a device on adb");
    
    print!("done!\nPushing APK... if nothing happens, double check the usb debugging prompt and accept it\n");
    adb_wait_process.wait().expect("Failed while waiting for device");
    let mut adb_push_process = Command::new(adbcmd)
        .args(&["push", "magiarecord.apk", phonepath])
        .spawn()
        .expect("Failed to push apk");
    adb_push_process.wait().expect("Failed while waiting for apk push");

    println!("done!\nInstalling APK\n");
    let mut adb_install_process = Command::new(adbcmd)
        .args(&["shell", "pm", "install", "-i", "\"com.android.vending\"", "-r", phonepath])
        .spawn()
        .expect("Failed to install apk");
    adb_install_process.wait().expect("Failed while waiting on install");
    
    println!("Killing adb server");
    let mut adb_kill_server = Command::new(adbcmd)
    .arg("kill-server")
    .spawn()
    .expect("Failed to kill server");
    adb_kill_server.wait().expect("Failled to kill adb server");
}
