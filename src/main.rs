extern crate chrono;
extern crate dirs;
extern crate gdk;
extern crate gdk_pixbuf;
extern crate gtk;
#[macro_use]
extern crate relm;
extern crate relm_attributes;
#[macro_use]
extern crate relm_derive;
extern crate rscam;

use self::Msg::*;
use chrono::Local;
use gdk_pixbuf::{PixbufLoader, PixbufLoaderExt};
use gtk::Orientation::Horizontal;
use gtk::Orientation::Vertical;
use gtk::{ButtonExt, GtkWindowExt, ImageExt, Inhibit, OrientableExt, WidgetExt};
use relm::{interval, Relm, Widget};
use relm_attributes::widget;
use rscam::{Camera, Config};
use std::process::Command;

use std::path::PathBuf;

pub struct Model {
    camera_status: Option<Camera>,
    camera_button_label: String,
}

#[derive(Msg)]
pub enum Msg {
    ScreenShotFull,
    ScreenShotArea,
    ToggleCamera,
    CloseCamera,
    CapturePixbuf,
    Quit,
    Redraw,
}

#[widget]
impl Widget for Win {
    fn model() -> Model {
        Model {
            camera_status: None,
            camera_button_label: String::from("Start Camera"),
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
                    None => self.model.camera_button_label = String::from("Start Camera"),
                }
            }
            CloseCamera => self.close_camera(),
            CapturePixbuf => self.capture_pixbuf(),
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

                    #[name="button_toggle_camera"]
                    gtk::Button {
                        clicked => ToggleCamera,
                        label: &self.model.camera_button_label,
                    },

                    #[name="button_close_camera"]
                    gtk::Button {
                        clicked => CloseCamera,
                        label: "Close Camera",
                    },

                    #[nema="button_capture_pixbuf"]
                    gtk::Button{
                        clicked => CapturePixbuf,
                        label: "Take a picture",
                    },
                },

                #[name="image"]
                gtk::Image {
                },
            },
            delete_event(_, _) => (Quit, Inhibit(false)),
        }
    }
}

impl Win {
    fn open_camera(&mut self) {
        let mut camera = Camera::new("/dev/video1").unwrap();
        camera
            .start(&Config {
                interval: (1, 30),
                resolution: (640, 360),
                format: b"MJPG",
                ..Default::default()
            })
            .unwrap();
        self.model.camera_status = Some(camera);
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
        if let Some(camera) = self.model.camera_status.as_mut() {
            let frame = camera.capture().unwrap();
            let image = &self.image;

            let loader = PixbufLoader::new();
            loader.write(&frame[..]).unwrap();
            loader.close().unwrap();

            let pixbuf = loader.get_pixbuf().unwrap();
            image.set_from_pixbuf(&pixbuf);
            while gtk::events_pending() {
                gtk::main_iteration_do(true);
            }
        }
    }

    fn capture_pixbuf(&mut self) {
        if let Some(_) = self.model.camera_status {
            let image = &self.image;
            let pixbuf = image.get_pixbuf().unwrap();

            let mut path = PathBuf::new();
            match dirs::home_dir() {
                Some(v) => path.push(v),
                None => path.push("."),
            }

            let time = Local::now();
            let filename = format!("{}{}", time.format("%Y-%m-%d-%H:%M:%S"), ".jpg");
            path.push("Pictures");
            path.push(filename);
            let filepath = path.to_str().unwrap();

            if let Err(_) = pixbuf.savev(filepath, "jpeg", &[("x-dpi", "640"), ("y-dpi", "360")]) {
                println!("Saving a picture failed.");
            }
        }
    }
}

fn main() {
    Win::run(()).expect("Win::run failed");
}
