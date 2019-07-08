#[macro_use]
extern crate relm;
#[macro_use]
extern crate relm_derive;

use self::Msg::*;
use chrono::Local;
use dirs::home_dir;
use gdk_pixbuf::{Pixbuf, PixbufLoader, PixbufLoaderExt};
use gtk::Orientation::Horizontal;
use gtk::Orientation::Vertical;
use gtk::{ButtonExt, GtkWindowExt, ImageExt, Inhibit, LabelExt, OrientableExt, WidgetExt};
use rayon::prelude::*;
use relm::{interval, Relm, Widget};
use relm_attributes::widget;
use rscam::{Camera, Config};
use std::path::PathBuf;
use std::process::Command;

pub struct Model {
    camera_found: bool,
    camera_device: &'static str,
    camera_status: Option<Camera>,
    camera_button_label: String,
    filter: Filters,
}

pub enum Filters {
    Normal,
    Gray,
    Reverse,
}

#[derive(Msg)]
pub enum Msg {
    ScreenShotFull,
    ScreenShotArea,
    ToggleCamera,
    CloseCamera,
    ChangeCamera,
    CapturePixbuf,
    ApplyNormal,
    ApplyGray,
    ApplyReverse,
    Redraw,
    Quit,
}

fn grayscale(pixbuf: &Pixbuf) {
    let n_channels = pixbuf.get_n_channels();
    let buf = unsafe { pixbuf.get_pixels() };

    buf.par_chunks_mut(n_channels as usize).for_each(|slice| {
        let gray = 0.299 * slice[0] as f32 + 0.587 * slice[1] as f32 + 0.114 * slice[2] as f32;
        slice[0] = gray as u8;
        slice[1] = gray as u8;
        slice[2] = gray as u8;
    });
}

fn reverse_rgb(pixbuf: &Pixbuf) {
    let buf = unsafe { pixbuf.get_pixels() };

    buf.par_chunks_mut(1).for_each(|x| {
        x[0] = 255 - x[0];
    })
}

#[widget]
impl Widget for Win {
    fn model() -> Model {
        Model {
            camera_found: false,
            camera_device: "/dev/video0",
            camera_status: None,
            camera_button_label: String::from("Start Camera"),
            filter: Filters::Normal,
        }
    }

    fn update(&mut self, event: Msg) {
        match event {
            ScreenShotFull => {
                Command::new("sh")
                    .arg("-c")
                    .arg("gnome-screenshot")
                    .spawn()
                    .expect("Failed to capture the screen.");
            }

            ScreenShotArea => {
                Command::new("sh")
                    .arg("-c")
                    .arg("gnome-screenshot --area")
                    .spawn()
                    .expect("Failed to capture the screen.");
            }

            ToggleCamera => {
                self.toggle_camera();
                match self.model.camera_status {
                    Some(_) => self.model.camera_button_label = String::from("Pause Camera"),
                    None => {
                        if self.model.camera_found {
                            self.model.camera_button_label = String::from("Start Camera")
                        } else {
                            self.model.camera_button_label = String::from("Cannot open camera!")
                        }
                    }
                }
            }

            CloseCamera => {
                self.close_camera();
                self.model.camera_button_label = String::from("Start Camera");
            }

            ChangeCamera => {
                self.change_camera();
            }

            CapturePixbuf => self.capture_pixbuf(),

            ApplyNormal => self.model.filter = Filters::Normal,

            ApplyGray => self.model.filter = Filters::Gray,

            ApplyReverse => self.model.filter = Filters::Reverse,

            Redraw => self.update_camera(),

            Quit => gtk::main_quit(),
        }
    }

    fn subscriptions(&mut self, relm: &Relm<Self>) {
        interval(relm.stream(), 10, || Redraw);
    }

    view! {
        gtk::Window {
            title: "Image Utils",
            gtk::Box {
                orientation: Horizontal,

                gtk::Box {
                    orientation: Vertical,

                    #[name="screen_label"]
                    gtk::Label {
                        text: "Screen Utils",
                    },

                    #[name="screenshot_full"]
                    gtk::Button {
                        clicked => ScreenShotFull,
                        label: "Screenshot(Full)"
                    },

                    #[name="screenshot_area"]
                    gtk::Button {
                        clicked => ScreenShotArea,
                        label: "Screenshot(Area)"
                    },

                    #[name="camera_label"]
                    gtk::Label {
                        text: "Camera Utils",
                    },

                    #[name="button_toggle_camera"]
                    gtk::Button {
                        clicked => ToggleCamera,
                        label: &self.model.camera_button_label,
                    },

                    #[name="button_change_camera"]
                    gtk::Button {
                        clicked => ChangeCamera,
                        label: "Change Camera",
                    },

                    #[name="button_close_camera"]
                    gtk::Button {
                        clicked => CloseCamera,
                        label: "Close Camera",
                    },

                    #[name="button_capture_pixbuf"]
                    gtk::Button {
                        clicked => CapturePixbuf,
                        label: "Take a picture",
                    },

                    #[name="label_filters"]
                    gtk::Label {
                        text: "Camera Filters",
                    },

                    #[name="button_to_normal"]
                    gtk::Button {
                        clicked => ApplyNormal,
                        label: "Normal",
                    },

                    #[name="button_to_gray"]
                    gtk::Button {
                        clicked => ApplyGray,
                        label: "Apply Grayscale",
                    },

                    #[name="button_to_reverse"]
                    gtk::Button {
                        clicked => ApplyReverse,
                        label: "Apply ReverseRGB",
                    },
                },

                gtk::Box {
                    orientation: Vertical,

                    #[name="image_label"]
                    gtk::Label {
                        text: "Camera Capture",
                    },

                    #[name="image"]
                    gtk::Image {
                    },
                },
            },
            delete_event(_, _) => (Quit, Inhibit(false)),
        }
    }
}

impl Win {
    fn open_camera(&mut self) {
        let mut camera = Camera::new(self.model.camera_device);
        match camera {
            Ok(ref mut cam) => match cam.start(&Config {
                interval: (1, 30),
                resolution: (640, 360),
                format: b"MJPG",
                ..Default::default()
            }) {
                Ok(_) => {
                    self.model.camera_status = Some(camera.unwrap());
                    self.model.camera_found = true
                }
                Err(_) => (),
            },
            Err(_) => (),
        }
    }

    fn stop_camera(&mut self) {
        self.model.camera_status = None;
    }

    fn toggle_camera(&mut self) {
        match self.model.camera_status {
            Some(_) => self.stop_camera(),
            None => self.open_camera(),
        }
    }

    fn close_camera(&mut self) {
        self.model.camera_status = None;
        let image = &self.image;
        image.clear();
    }

    fn update_camera(&mut self) {
        match self.model.camera_status {
            Some(ref mut camera) => match camera.capture() {
                Ok(frame) => {
                    let image = &self.image;
                    let loader = PixbufLoader::new();
                    loader.write(&frame[..]).unwrap();

                    match loader.close() {
                        Ok(_) => {
                            let pixbuf = loader.get_pixbuf().unwrap();

                            match self.model.filter {
                                Filters::Gray => grayscale(&pixbuf),
                                Filters::Reverse => reverse_rgb(&pixbuf),
                                _ => (),
                            }

                            image.set_from_pixbuf(&pixbuf);
                            while gtk::events_pending() {
                                gtk::main_iteration_do(true);
                            }
                        }
                        Err(_) => self.close_camera(),
                    }
                }
                Err(_) => self.close_camera(),
            },
            None => (),
        }
    }

    fn change_camera(&mut self) {
        let out = Command::new("sh")
            .arg("-c")
            .arg("ls /dev/ | grep -c video")
            .output()
            .expect("failed to search camera devices.");

        let device_out = out.stdout;
        let device_str = std::str::from_utf8(&device_out).unwrap();
        if device_str == "3" {
            if self.model.camera_device == "/dev/video0" {
                self.stop_camera();
                self.model.camera_device = "/dev/video1";
                self.open_camera();
            } else {
                self.stop_camera();
                self.model.camera_device = "/dev/video0";
                self.open_camera();
            }
        }
    }

    fn capture_pixbuf(&mut self) {
        if let Some(_) = self.model.camera_status {
            let image = &self.image;
            let pixbuf = image.get_pixbuf().unwrap();

            let mut path = PathBuf::new();
            match home_dir() {
                Some(v) => path.push(v),
                None => path.push("."),
            }

            let time = Local::now();
            let filename = format!("{}{}", time.format("%Y-%m-%d-%H:%M:%S"), ".jpg");
            path.push("Pictures");
            path.push(filename);
            let filepath = path.to_str().unwrap();

            if let Err(_) = pixbuf.savev(filepath, "jpeg", &[("x-dpi", "1280"), ("y-dpi", "720")]) {
                println!("Saving a picture failed.");
            }
        }
    }
}

fn main() {
    Win::run(()).expect("Win::run failed");
}
