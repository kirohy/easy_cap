use gdk_pixbuf::Pixbuf;
use rand::distributions::Uniform;
use rand::{thread_rng, Rng};
use rand_distr::{Distribution, Normal};
use rayon::prelude::*;

pub fn grayscale(pixbuf: &Pixbuf) {
    let n_channels = pixbuf.get_n_channels();
    let buf = unsafe { pixbuf.get_pixels() };

    buf.par_chunks_mut(n_channels as usize).for_each(|slice| {
        let gray = 0.299 * slice[0] as f32 + 0.587 * slice[1] as f32 + 0.114 * slice[2] as f32;
        slice[0] = gray as u8;
        slice[1] = gray as u8;
        slice[2] = gray as u8;
    });
}

pub fn reverse_rgb(pixbuf: &Pixbuf) {
    let buf = unsafe { pixbuf.get_pixels() };

    buf.par_chunks_mut(1).for_each(|x| {
        x[0] = 255 - x[0];
    })
}

#[derive(Copy, Clone)]
struct Particle {
    pos: (i32, i32),
    vel: (i32, i32),
    like: f32,
    wgt: f32,
    keep: bool,
}

impl Particle {
    fn new(p: (i32, i32), v: (i32, i32), l: f32, w: f32, k: bool) -> Particle {
        Particle {
            pos: p,
            vel: v,
            like: l,
            wgt: w,
            keep: k,
        }
    }
}

#[derive(Copy, Clone)]
pub struct Rgb {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl Rgb {
    pub fn new(r: u8, g: u8, b: u8) -> Rgb {
        Rgb {
            red: r,
            green: g,
            blue: b,
        }
    }
    fn calc_like(&self, target: Rgb) -> f32 {
        let diff_r = (self.red as f32 - target.red as f32).powi(2);
        let diff_g = (self.green as f32 - target.green as f32).powi(2);
        let diff_b = (self.blue as f32 - target.blue as f32).powi(2);
        1. - (diff_r + diff_g + diff_b).sqrt() / (255.0_f32 * 3.0_f32.sqrt())
    }
}

pub struct ParamsForParticle {
    camera: (i32, i32),
    num: u32,
    center: (i32, i32),
    pub target_rgb: Rgb,
    pars: Vec<Particle>,
}

impl ParamsForParticle {
    pub fn new(c: (i32, i32), n: u32, t: Rgb) -> ParamsForParticle {
        let mut pars_tmp: Vec<Particle> = Vec::new();
        let mut rng = thread_rng();
        for _ in 0..rng.gen_range(0, n) {
            let pt: (i32, i32) = (
                rng.sample(Uniform::new(0, c.0)),
                rng.sample(Uniform::new(0, c.1)),
            );
            let p: Particle = Particle::new(pt, (0_i32, 0_i32), 1., 0., false);
            pars_tmp.push(p);
        }
        ParamsForParticle {
            camera: c,
            num: n,
            center: (c.0 / 2, c.1 / 2),
            target_rgb: t,
            pars: pars_tmp,
        }
    }
}

fn pixbuf_to_vec(pixbuf: &Pixbuf, params: &ParamsForParticle, mat: &mut Vec<Vec<Rgb>>) {
    let buf = unsafe { pixbuf.get_pixels() };
    mat.clear();
    for iter_all in buf.chunks((params.camera.0 * 3) as usize) {
        let mut tmp_vec = Vec::new();
        for iter_one in iter_all.chunks(3) {
            let pixel = Rgb {
                red: iter_one[0],
                green: iter_one[1],
                blue: iter_one[2],
            };
            tmp_vec.push(pixel);
        }
        mat.push(tmp_vec);
    }
}

pub fn particle(pixbuf: &Pixbuf, params: &mut ParamsForParticle) {
    let mut buf_vec: Vec<Vec<Rgb>> = Vec::new();
    pixbuf_to_vec(pixbuf, params, &mut buf_vec);

    let target_rgb = params.target_rgb;

    for iter in params.pars.iter_mut() {
        iter.pos.0 += iter.vel.0;
        iter.pos.1 += iter.vel.1;
        if 0 < iter.pos.0
            && iter.pos.0 < params.camera.0
            && 0 < iter.pos.1
            && iter.pos.1 < params.camera.1
        {
            let rgb = buf_vec[iter.pos.1 as usize][iter.pos.0 as usize];
            iter.like = rgb.calc_like(target_rgb);
        } else {
            iter.like = 0.;
        }
    }

    params
        .pars
        .par_sort_by(|a, b| a.like.partial_cmp(&b.like).unwrap());

    let length = params.pars.len();
    let thresh_like = 0.9_f32;
    let thresh_keep = length / 100;

    for (i, iter) in params.pars.iter_mut().enumerate() {
        if iter.like > thresh_like || i > (length - thresh_keep) {
            iter.keep = true;
        } else {
            iter.keep = false;
        }
    }

    params.pars.retain(|&x| x.keep == true);

    let mut like_sum: f32 = 0.;
    for iter in params.pars.iter() {
        like_sum += iter.like;
    }

    params
        .pars
        .par_iter_mut()
        .for_each(|x| x.wgt = x.like / like_sum);

    let mut pars_new: Vec<Particle> = Vec::new();
    let normal = Normal::new(0., (params.camera.0 + params.camera.1) as f32).unwrap();

    for iter in params.pars.iter_mut() {
        let num_new = (iter.wgt * (params.num - length as u32) as f32) as usize;
        for _ in 0..num_new {
            let radius = normal.sample(&mut rand::thread_rng()) * (1. - iter.like);
            let theta = rand::thread_rng()
                .sample(Uniform::new(-std::f32::consts::PI, std::f32::consts::PI));
            let v = ((radius * theta.cos()) as i32, (radius * theta.sin()) as i32);
            let pt = (v.0 + iter.pos.0 as i32, v.1 + iter.pos.1 as i32);
            let p = Particle::new(pt, v, iter.like, iter.wgt, false);
            pars_new.push(p);
        }
    }

    params.pars.append(&mut pars_new);

    for iter in params.pars.iter_mut() {
        if 0 < iter.pos.0
            && iter.pos.0 < params.camera.0
            && 0 < iter.pos.1
            && iter.pos.1 < params.camera.1
        {
            pixbuf.put_pixel(iter.pos.0, iter.pos.1, 255, 0, 0, 0);
        }
        params.center.0 += iter.pos.0;
        params.center.1 += iter.pos.1;
    }

    let length = params.pars.len();

    if length > 0 {
        params.center.0 /= length as i32;
        params.center.1 /= length as i32;

        if params.center.1 < params.camera.1 {
            for i in 0..params.camera.0 {
                pixbuf.put_pixel(i, params.center.1, 255, 255, 255, 0);
            }
        }

        if params.center.0 < params.camera.0 {
            for i in 0..params.camera.1 {
                pixbuf.put_pixel(params.center.0, i, 255, 255, 255, 0);
            }
        }
    }
}
