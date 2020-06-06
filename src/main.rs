use self::Msg::*;
use chrono::Local;
use dirs::home_dir;
use easy_cap::filters;
use gdk_pixbuf::{PixbufLoader, PixbufLoaderExt};
use gtk::Orientation::Horizontal;
use gtk::Orientation::Vertical;
use gtk::{
    ButtonExt, EditableSignals, EntryExt, GtkWindowExt, ImageExt, Inhibit, LabelExt, OrientableExt,
    WidgetExt,
};
use relm::{interval, Relm, Widget};
use relm_derive::{widget, Msg};
use rscam::{Camera, Config};
use std::path::PathBuf;
use std::process::Command;

#[derive(Copy, Clone, PartialEq)]
pub enum FilterType {
    Normal,
    Gray,
    Reverse,
    Particle,
}

pub struct Model {
    camera_found: bool,
    camera_device: &'static str,
    camera_status: Option<Camera>,
    camera_button_label: String,
    filter: FilterType,
    filter_prev: FilterType,
    particles: filters::ParamsForParticle,
    target_rgb: filters::Rgb,
}

#[derive(Msg)]
pub enum Msg {
    ToggleCamera,
    CloseCamera,
    ChangeCamera,
    CapturePixbuf,
    ApplyNormal,
    ApplyGray,
    ApplyReverse,
    ApplyParticle,
    ChangeRed(String),
    ChangeGreen(String),
    ChangeBlue(String),
    Redraw,
    Quit,
}

#[widget]
impl Widget for Win {
    fn model() -> Model {
        Model {
            camera_found: false,
            camera_device: "/dev/video0",
            camera_status: None,
            camera_button_label: String::from("Start Camera"),
            filter: FilterType::Normal,
            filter_prev: FilterType::Normal,
            particles: filters::ParamsForParticle::new(
                (640, 360),
                10000,
                filters::Rgb::new(0, 0, 0),
            ),
            target_rgb: filters::Rgb::new(0, 0, 0),
        }
    }

    fn update(&mut self, event: Msg) {
        match event {
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

            ApplyNormal => self.model.filter = FilterType::Normal,

            ApplyGray => self.model.filter = FilterType::Gray,

            ApplyReverse => self.model.filter = FilterType::Reverse,

            ApplyParticle => self.model.filter = FilterType::Particle,

            ChangeRed(red_string) => {
                let red = red_string.parse::<u8>();
                if let Ok(v) = red {
                    self.model.particles.target_rgb.red = v;
                    self.model.target_rgb.red = v;
                }
            }

            ChangeGreen(green_string) => {
                let green = green_string.parse::<u8>();
                if let Ok(v) = green {
                    self.model.particles.target_rgb.green = v;
                    self.model.target_rgb.green = v;
                }
            }

            ChangeBlue(blue_string) => {
                let blue = blue_string.parse::<u8>();
                if let Ok(v) = blue {
                    self.model.particles.target_rgb.blue = v;
                    self.model.target_rgb.blue = v;
                }
            }

            Redraw => self.update_camera(),

            Quit => gtk::main_quit(),
        }
    }

    fn subscriptions(&mut self, relm: &Relm<Self>) {
        interval(relm.stream(), 10, || Redraw);
    }

    view! {
        gtk::Window {
            title: "Easy Capture",
            gtk::Box {
                orientation: Horizontal,

                gtk::Box {
                    orientation: Vertical,

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

                    #[name="button_to_particle"]
                    gtk::Button {
                        clicked => ApplyParticle,
                        label: "Apply ParticleFilter",
                    },

                    #[name="target_red"]
                    gtk::Entry {
                        changed(entry) => {
                            let red_string = entry.get_text().expect("failed to parse").to_string();
                            ChangeRed(red_string)
                        },
                        placeholder_text: Some("Enter Red value"),
                    },

                    #[name="target_green"]
                    gtk::Entry {
                        changed(entry) => {
                            let green_string = entry.get_text().expect("failed to parse").to_string();
                            ChangeGreen(green_string)
                        },
                        placeholder_text: Some("Enter Green value"),
                    },

                    #[name="target_blue"]
                    gtk::Entry {
                        changed(entry) => {
                            let blue_string = entry.get_text().expect("failed to parse").to_string();
                            ChangeBlue(blue_string)
                        },
                        placeholder_text: Some("Enter blue value"),
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

                    #[name="quit_button"]
                    gtk::Button {
                        clicked => Quit,
                        label: "Quit",
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

                            if self.model.filter == FilterType::Particle
                                && self.model.filter_prev != FilterType::Particle
                            {
                                self.model.particles = filters::ParamsForParticle::new(
                                    (640, 360),
                                    10000,
                                    self.model.target_rgb,
                                );
                            }

                            match self.model.filter {
                                FilterType::Gray => filters::grayscale(&pixbuf),
                                FilterType::Reverse => filters::reverse_rgb(&pixbuf),
                                FilterType::Particle => {
                                    filters::particle(&pixbuf, &mut self.model.particles)
                                }
                                _ => (),
                            }

                            image.set_from_pixbuf(Some(&pixbuf));
                            self.model.filter_prev = self.model.filter;

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
        if device_str != "2" {
            if self.model.camera_device == "/dev/video0" {
                self.close_camera();
                self.model.camera_device = "/dev/video2";
                self.open_camera();
            } else {
                self.close_camera();
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
