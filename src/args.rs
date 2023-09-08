use log::info;
use screenshots::{self, Screen};
use std::fs;
use winit::dpi::PhysicalPosition;
mod image_proc;
use crate::selection_tool;

pub fn parse(args: Vec<String>) {
    info!("Arguments: {:?}", args);
    let help = "RTFM";
    let version = env!("CARGO_PKG_VERSION");

    if args.is_empty() { return }

    // DONE: replace outer iter and move it into inner iters
    let mut j = 0;
    while j < args.len() {
        match &args[j][..] {
            "--version" | "-v" => println!("{}", version),
            "--help" | "-h" => println!("{}", help),

            // Optional gui flag jsut for ocd ppl
            "--gui" => {
                info!("GUI mode");
                let screens = Screen::all().unwrap();
                let mut pos = PhysicalPosition::new(0.0, 0.0);
                let mut br = PhysicalPosition::new(0.0, 0.0);
                for screen in screens {
                    let screen_pos = PhysicalPosition::new(
                        screen.display_info.x as f64,
                        screen.display_info.y as f64,
                    );
                    let screen_br = PhysicalPosition::new(
                        screen_pos.x + screen.display_info.width as f64,
                        screen_pos.y + screen.display_info.height as f64,
                    );
                    if pos.x > screen_pos.x {
                        pos.x = screen_pos.x;
                    }
                    if pos.y > screen_pos.y {
                        pos.y = screen_pos.y;
                    }
                    if br.x < screen_br.x {
                        br.x = screen_br.x;
                    }
                    if br.y < screen_br.y {
                        br.y = screen_br.y;
                    }
                }

                info!("{:?}\n{:?}", pos, br);
                selection_tool::run(pos, br);
            }

            // text extraction mode
            // TODO: WIP
            "--text" | "-T" => {
                println!("AI text extraction mode enabled");
                // TODO: add AI functionality
                if args.len() > 1 {
                    let mut i = j + 1;
                    while i < args.len() {
                        match &args[i][..] {
                            "-o" => {
                                println!("Output to file {}", &args[i + 1][..]);
                                i += 1;
                            }
                            "-v" => {
                                println!("Verbose");
                            }
                            "-cp" => println!("Copy to clipboard"),
                            "-t" => {
                                println!("Wait {} seconds", &args[i + 1][..]);
                                i += 1;
                            }
                            _ => {
                                j -= 1;
                                break;
                            }
                        }
                        i += 1;
                    }
                }
            }

            // image / normal sc flag
            "--image" | "-I" => {
                println!("Image mode enabled");
                // DONE: add screenshot functionality

                if args.len() > 1 {
                    let mut i = j + 1;
                    while i < args.len() {
                        match &args[i][..] {
                            "--output" | "-o" => {
                                let points;
                                // TODO: harden this
                                if i + 2 < args.len() {
                                    points = (
                                        Some(image_proc::Point {
                                            x: args[i + 1][..].parse::<i32>().unwrap(),
                                            y: args[i + 2][..].parse::<i32>().unwrap(),
                                        }),
                                        Some(image_proc::Point {
                                            x: args[i + 3][..].parse::<i32>().unwrap(),
                                            y: args[i + 4][..].parse::<i32>().unwrap(),
                                        }),
                                    );
                                    i += 4;
                                    j = i;
                                } else {
                                    points = (None, None);
                                }
                                let compressed_images = image_proc::run(None, points);
                                for (k, images) in compressed_images.into_iter().enumerate() {
                                    // TODO: make option and unwrap or for default file location
																		// fuck this code
                                    fs::write(format!("target/{}.png", k), images).unwrap();
                                }
                                i += 1;
                            }
                            "--clipboard" | "-cp" => println!("Copy to clipboard"),
                            "-t" => {
                                println!("Wait {} seconds", &args[i + 1][..]);
                                i += 1;
                            }
                            _ => {
                                j -= 1;
                                break;
                            }
                        }
                        i += 1;
                    }
                }
            }
            "--display-info" => {
                let screens = Screen::all().unwrap();
                for screen in screens {
                    println!("{screen:?}");
                }
            }

            // all the other cases
            _ => {
                if j != args.len() - 1 {
                    println!("pictura: invalid mode {}", args[j])
                } else {
                    info!("Executed successfully!");
                }
            }
        }

        j += 1;
    }
}

#[allow(dead_code)]
pub fn capture(app: (PhysicalPosition<f64>, PhysicalPosition<f64>)) {
    let points = (
        Some(image_proc::Point {
            x: app.0.x as i32,
            y: app.0.y as i32,
        }),
        Some(image_proc::Point {
            x: app.1.x as i32,
            y: app.1.y as i32,
        }),
    );
    let compressed_images = image_proc::run(None, points);
    for (k, images) in compressed_images.into_iter().enumerate() {
        fs::write(format!("target/{}.png", k), images).unwrap();
    }
}
