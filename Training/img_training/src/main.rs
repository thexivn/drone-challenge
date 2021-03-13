extern crate opencv;

use opencv::core;
use opencv::core::Mat;
use opencv::imgcodecs;
use opencv::imgproc;
use opencv::objdetect;
use std::{fs, ops::Add, ops::Sub, str::from_utf8};
use std::{
    io::prelude::*,
    net::{SocketAddr, UdpSocket},
};
//use std::net::{SocketAddr, UdpSocket};
use opencv::prelude::*;
use opencv::videoio::VideoCapture;

fn main() {
    let _dir = "../Img/tor/";
    let dest = "../Img/neg960/";

    let args: Vec<String> = std::env::args().collect();
    println!("{:?}", args);
    if args[1] == "add" {
        resize_img_gscale("../Img/pos/", "../Img/pos960/", 0);
    } else if args[1] == "txt" {
        create_bg_txt(dest);
    } else if args[1] == "cam" {
        cam();
    }
}

fn resize_img_gscale(dir: &str, dest: &str, done: usize) {
    for (i, entry) in fs::read_dir(dir).expect("dir").enumerate() {
        let img_path = entry.expect("entry");
        println!("img {} - {:?}", i, img_path);
        let img = imgcodecs::imread(img_path.path().to_str().unwrap(), imgcodecs::IMREAD_COLOR)
            .expect("img");
        let mut res = Mat::default().unwrap();
        imgproc::resize(
            &img,
            &mut res,
            core::Size::new(1280, 960),
            0f64,
            0f64,
            imgproc::INTER_LINEAR,
        )
        .unwrap();

        imgcodecs::imwrite(
            format!("{}img-{}.jpg", dest, i + done).as_str(),
            &res,
            &core::Vector::new(),
        )
        .unwrap();
    }
}

fn create_bg_txt(dir: &str) {
    let mut file = fs::File::create("bg.txt").expect("creation failed");
    for entry in fs::read_dir(dir).unwrap() {
        if let Ok(path) = entry {
            let path = format!("{}\n", path.path().to_str().unwrap());
            let bytes = path.into_bytes();
            file.write_all(&bytes).unwrap();
        }
    }
}

fn cam() {
    let mut red_cs = objdetect::CascadeClassifier::new("CS_RED_25.xml").unwrap();
    let color_red = core::Scalar::new(0.0, 0.0, 255.0, 0.0);
    let color_cyn = core::Scalar::new(252.0, 187.0, 22.0, 0.0);
    let font = opencv::highgui::font_qt(
        "Times",
        12,
        color_cyn,
        opencv::highgui::QT_FONT_BOLD,
        opencv::highgui::QT_STYLE_NORMAL,
        0,
    )
    .unwrap();

    let tello_address = SocketAddr::from(([192, 168, 10, 1], 8889));
    let socket = UdpSocket::bind("192.168.10.2:9000").expect("Failed to create socketd");
    socket
        .send_to(b"command", tello_address)
        .expect("Failed to start sdk");
    let mut buffer = [0; 2048];
    let size = socket
        .recv(&mut buffer)
        .expect("Failed to receive response");
    println!("{:?}", std::str::from_utf8(&buffer[..size]));

    opencv::highgui::named_window("tello", 1).expect("f the window");
    socket.send_to(b"streamon", tello_address).unwrap();
    socket.recv(&mut buffer).expect("failed");
    let mut video = VideoCapture::from_file(
        "udp://0.0.0.0:11111?overrun_nonfatal=1&fifo_size=50000000",
        //"https://192.168.1.3:4343/video?overrun_nonfatal=1&fifo_size=50000000",
        opencv::videoio::CAP_FFMPEG,
    )
    .unwrap();
    let mut frame = Mat::default().unwrap();
    let mut rects: core::Vector<core::Rect> = core::Vector::new();
    let mut instant = std::time::Instant::now();
    video.read(&mut frame).expect("frame");
    let screen_rect = core::Rect::new(
        0,
        0,
        video.get(opencv::videoio::CAP_PROP_FRAME_WIDTH).unwrap() as i32,
        video.get(opencv::videoio::CAP_PROP_FRAME_HEIGHT).unwrap() as i32,
    );
    let mut take_off = false;
    let mut fails = 50;
    socket.set_nonblocking(true).unwrap();
    socket.send_to(b"takeoff", tello_address).unwrap();

    loop {
        if video.read(&mut frame).unwrap() {
            if !take_off {
                match socket.recv_from(&mut buffer) {
                    Ok(n) => {
                        let read = std::str::from_utf8(&buffer[..n.0]).unwrap();
                        if read == "Ok" {
                            take_off = true;
                        } else {
                            socket.send_to(b"land", tello_address).unwrap();
                            break;
                        }
                    }
                    Err(ref r) if r.kind() == std::io::ErrorKind::WouldBlock => {
                        println!("wait");
                    }
                    Err(e) => {
                        eprintln!("{}", e);
                        break;
                    }
                }
            } else {
                //imgproc::resize(&frame.clone(), &mut frame, core::Size::new(250,250),0f64,0f64, imgproc::INTER_LINEAR).unwrap();
                //imgproc::cvt_color(&frame.clone(), &mut frame, imgproc::COLOR_RGB2GRAY, 0).unwrap();
                if instant.elapsed() >= std::time::Duration::from_millis(200) {
                    red_cs
                        .detect_multi_scale(
                            &frame,
                            &mut rects,
                            1.3,
                            5,
                            0,
                            core::Size::new(30, 30),
                            core::Size::new(0, 0),
                        )
                        .expect("Faccia");
                    instant = std::time::Instant::now();
                }

                let command: String;
                if *&rects.len() > 0usize {
                    if let Ok(cs) = &rects.get(0usize) {
                        imgproc::rectangle(&mut frame, *cs, color_red, 2, imgproc::LINE_8, 0)
                            .unwrap();
                        let rc = rect_center(&cs);
                        imgproc::circle(&mut frame, rc, 1, color_red, 4, imgproc::LINE_8, 0)
                            .unwrap();
                        imgproc::line(
                            &mut frame,
                            rc,
                            rect_center(&screen_rect),
                            color_red,
                            2,
                            imgproc::LINE_8,
                            0,
                        )
                        .unwrap();
                        command = rc_command(&cs, &screen_rect, 7);
                    } else {
                        command = "rc 0 0 0 0".to_string();
                    }
                    fails = 50;
                } else {
                    command = "rc 0 0 0 0".to_string();
                    fails -= 1;
                    if fails == 0 {
                        break;
                    }
                }
                if let Err(a) = opencv::highgui::add_text(
                    &mut frame,
                    command.as_str(),
                    rect_center(&screen_rect).add(core::Point::new(-50, 19)),
                    &font,
                ) {
                    eprintln!("{}", a);
                }

                imgproc::circle(
                    &mut frame,
                    rect_center(&screen_rect),
                    7,
                    core::Scalar::new(231.0, 4.0, 239.0, 0.0),
                    1,
                    imgproc::LINE_8,
                    0,
                )
                .unwrap();
                opencv::highgui::imshow("tello", &frame).expect("poop");
            }
        }
        if let Ok(key) = opencv::highgui::wait_key(2) {
            if key == 27 {
                println!("{}", key);
                break;
            }
        }
    }

    socket.set_nonblocking(false).unwrap();
    socket.send_to(b"land", tello_address).unwrap();
    socket.recv(&mut buffer).unwrap();
}

#[inline]
fn rect_center(rect: &core::Rect) -> core::Point {
    core::Point::new(
        rect.size().width / 2 + rect.tl().x,
        rect.size().height / 2 + rect.tl().y,
    )
}

fn rc_command(rect_track: &core::Rect, space: &core::Rect, radius: u32) -> String {
    let center_track = rect_center(&rect_track);
    let tr: core::Point = center_track.sub(rect_center(space));

    let vx = (100 * tr.x) / ((space.size().width as f64 / 2.0) as i32);
    let vy = (100 * tr.y) / ((space.size().height as f64 / 2.0) as i32);

    let r_area = rect_track.area() as f64 / space.area() as f64;

    format!(
        "rc {} {} {} 0",
        if vx.abs() as u32 <= radius { 0 } else { vx },
        if r_area <= 0.005 {
            -15
        } else {
            if r_area >= 0.02 {
                15
            } else {
                0
            }
        },
        if vy.abs() as u32 <= radius { 0 } else { vy }
    )
}
