extern crate sdl2;

use std::env;
use std::path::Path;
use sdl2::image::{ImageRWops, LoadTexture, INIT_PNG, INIT_JPG};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rwops::RWops;
use sdl2::surface::Surface;
use	std::time::{Instant};
use std::thread;
use std::time::Duration;
use std::process::Command;
use sdl2::render::{WindowCanvas};

//下载SDL_image
//https://www.libsdl.org/projects/SDL_image/

const WINDOW_WIDTH:u32 = 360;
const WINDOW_HEIGHT:u32 = 640;

struct Holder{
    point_one:Option<(i32, i32)>,
    point_two:Option<(i32, i32)>
}

pub fn main() {
    println!("++++++++++++++++++");
    println!("按F5初始化并刷新屏幕");
    println!("++++++++++++++++++");
    let mut adb_ok = check_adb();

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let _image_context = sdl2::image::init(INIT_PNG | INIT_JPG).unwrap();
    let window = video_subsystem.window("微信跳一跳助手", WINDOW_WIDTH, WINDOW_HEIGHT)
      .position_centered()
      .build()
      .unwrap();

    let mut canvas = window.into_canvas().software().build().unwrap();
    if adb_ok {
        screencap(&mut canvas);
    }

    let mut holder = Holder{
        point_one:None,
        point_two:None
    };

    'mainloop: loop {
            for event in sdl_context.event_pump().unwrap().poll_iter() {
                match event {
                    Event::Quit{..} |
                    Event::KeyDown {keycode: Option::Some(Keycode::Escape), ..} =>
                        break 'mainloop,
                    Event::KeyDown {keycode: Option::Some(Keycode::F5), ..} =>{
                        adb_ok = check_adb();
                        holder.point_one = None;
                        if adb_ok {
                            screencap(&mut canvas);
                        }
                    }
                    Event::MouseButtonUp {x, y, ..} =>{
                        if holder.point_one == None {
                            holder.point_one = Some((x, y));
                            println!("起跳点:({},{})", x, y);
                        }else{
                            holder.point_two = Some((x, y));
                            println!("结束点:({},{})", x, y);
                            let (x1, y1) = holder.point_one.unwrap();
                            let (x2, y2) = holder.point_two.unwrap();
                            //计算距离
                            let distance = (((x1-x2)*(x1-x2)+(y1-y2)*(y1-y2)) as f32).sqrt().abs();
                            
                            //距离按照屏幕宽度比例
                            let distance = distance/WINDOW_WIDTH as f32;

                            //根据距离模拟按下事件
                            let second = distance*1.47; //距离*2=按下秒数

                            execute(&format!("adb shell input swipe 320 410 320 410 {}", (second*1000.0) as i32 ));
                            println!("距离:{}, 按下时间:{}ms", distance, second*1000.0);

                            //清空
                            holder.point_one = None;
                            holder.point_two = None;
                            //延迟刷新屏幕
                            thread::sleep(Duration::from_millis(600));
                            //刷新屏幕
                            screencap(&mut canvas);
                        }
                    }
                    _ => {}
                }
            }
    }
}

fn check_adb()->bool{
    let adb_devices = execute_result("adb devices");
    println!("{}", adb_devices);
    let mut lines = adb_devices.lines();
    let first_line = lines.next();
    let ok = match first_line{
        Some("List of devices attached")=>{
            if lines.count() == 1{
                false
            }else{
                true
            }
        }
        _ =>{
            false
        }
    };
    if !ok{
        println!("adb设备读取失败：\n1、检查是否配置adb环境变量。\n2、检查手机是否链接电脑，并打开调试模式。");
    }
    ok
}

//adb截屏
fn screencap(canvas:&mut WindowCanvas){
    let texture_creator = canvas.texture_creator();
    let capture:Vec<u8> = execute("adb exec-out screencap -p");//执行adb截图命令
    let rwops = RWops::from_bytes(capture.as_slice()).unwrap();
    let surface = rwops.load_png().unwrap();
    let texture = texture_creator.create_texture_from_surface(surface).unwrap();
    canvas.copy(&texture, None, None).expect("Render failed");
    canvas.present();
}

fn print_elapsed(tag:i32, start:&mut Instant){
    println!("{}>>耗时{:?}毫秒.", tag, start.elapsed().as_secs()*1000+(start.elapsed().subsec_nanos()/1000000u32) as u64);
    *start = Instant::now();
}

//执行shell
pub fn execute<'a>(cmd:&'a str) ->Vec<u8>{
    //执行命令
    let output = if cfg!(target_os = "windows") {
        Command::new("cmd")
                .args(&["/C", cmd])
                .output()
                .expect("failed to execute process")
    } else {
        Command::new("sh")
                .arg("-c")
                .arg(cmd)
                .output()
                .expect("failed to execute process")
    };
    output.stdout
}

pub fn execute_result<'a>(cmd:&'a str) ->String{
    let result = execute(cmd);
    String::from_utf8(result).unwrap_or_else(|err|{
        println!("{:?}", err);
        String::from("")
    })
}