use bladerf;
use std::vec::Vec;
use std::sync::Arc;
use std::thread;
use num_complex::Complex;
use std::sync::mpsc::channel;
use std::time::Duration;

fn multiply_slice(a: &[Complex<f64>], b: &[Complex<f64>], c: &mut [Complex<f64>]) {
    for ((va, vb), vc) in a.iter().zip(b.iter()).zip(c.iter_mut()) {
        *vc = va * vb;
    }
}

fn conjugate_slice(a: &mut [Complex<f64>]) {
    for va in a.iter_mut() {
        *va = va.conj();
    }
}

fn sum_slice(a: &[Complex<f64>]) -> Complex<f64> {
    let mut out = Complex::<f64> {
        re: 0.0,
        im: 0.0,
    };

    for va in a.iter() {
        out.re += va.re;
        out.im += va.im;
    }

    out
}

fn convert_iqi16_to_iqf64(v: &Vec<Complex<i16>>) -> Vec<Complex<f64>> {
    let mut out: Vec<Complex<f64>> = Vec::with_capacity(v.len());
    for vv in v.iter() {
        out.push(Complex::<f64> {
            re: vv.re as f64,
            im: vv.im as f64,
        });
    }

    out
}

fn main() {
    println!("opening bladerf..");
    let dev = bladerf::open(None).expect("card should have been opened");
    println!(
        "fpga:{:?} firmware:{:?}",
        dev.fpga_version().expect("should have returned fpga version"),
        dev.fw_version().expect("should have returned firmware version")
    );

    let freq = 3000000000u64;

    dev.set_frequency(bladerf::bladerf_module::RX0, freq).expect(
        "should have set the RX frequency"
    );
    dev.set_frequency(bladerf::bladerf_module::TX0, freq).expect(
        "should have set the TX frequency"
    );

    dev.set_gain(bladerf::bladerf_module::TX0, 0).expect("TX0 gain set");
    dev.set_gain(bladerf::bladerf_module::TX1, 0).expect("TX1 gain set");
    dev.set_gain(bladerf::bladerf_module::RX0, 60).expect("RX0 gain set");
    dev.set_gain(bladerf::bladerf_module::RX1, 60).expect("RX1 gain set");

    let sps = 520834u32;

    dev.set_sample_rate(bladerf::bladerf_module::RX0, 520834).expect(
        "RX0/RX1 sampling rate set"
    );

    dev.set_sample_rate(bladerf::bladerf_module::TX0, 520834).expect(
        "TX0/TX1 sampling rate set"
    );

    let num_buffers = 16u32;
    let buffer_size = 4096u32;
    let num_transfers = 8u32;
    let stream_timeout = 20u32;

    dev.sync_config(
        bladerf::bladerf_channel_layout::RX_X1,
        bladerf::bladerf_format::SC16_Q11,
        num_buffers, buffer_size,
        Some(num_transfers),
        stream_timeout
    ).expect("sync_config for rx");

    dev.sync_config(
        bladerf::bladerf_channel_layout::TX_X1,
        bladerf::bladerf_format::SC16_Q11,
        num_buffers, buffer_size,
        Some(num_transfers),
        stream_timeout
    ).expect("sync_config for tx");

    dev.enable_module(bladerf::bladerf_module::RX0, true).expect(
        "rx0 module enable"
    );
    dev.enable_module(bladerf::bladerf_module::TX0, true).expect(
        "tx0 module enable"
    );

    let samps: usize = 4096;
    let mut signal: Vec<Complex<f64>> = vec!(Complex::<f64> { re: 0.0f64, im: 0.0f64 }; samps);
    let mut tx_data: Vec<Complex<i16>> = vec!(Complex::<i16> { re: 0i16, im: 0i16 }; samps);
    let local_freq = 10e3f64;
    let theta_step = local_freq * std::f64::consts::PI * 2.0f64 / (sps as f64);

    {
        let mut theta = 0.0f64;
        for x in 0..samps {
            signal[x].re = f64::cos(theta);
            signal[x].im = f64::sin(theta);
            tx_data[x].re = (signal[x].re * 2000.0) as i16;
            tx_data[x].im = (signal[x].im * 2000.0) as i16;
            theta += theta_step;
        }
    }

    let dev_arc = Arc::new(dev);

    let dev_arc_tx = dev_arc.clone();

    let (rx_tx, tx_rx) = channel::<bool>();

    let tx_handler = thread::spawn(move || {
        loop {
            dev_arc_tx.sync_tx(&tx_data, None, 20000).expect("tx sync call");
            match tx_rx.recv_timeout(Duration::from_millis(0)) {
                Ok(_) => break,
                Err(_) => (),
            };
        }
    });

    let mut rx_data: Vec<Complex<i16>> = vec!(Complex::<i16> { re: 0i16, im: 0i16 }; samps);

    conjugate_slice(&mut signal);

    let mut tmp: Vec<Complex<f64>> = vec!(Complex::<f64> {
        re: 0.0,
        im: 0.0,
    }; samps);

    for _ in 0..100 {
        dev_arc.sync_rx(&mut rx_data, None, 20000).expect("rx sync call");
        let rx_signal = convert_iqi16_to_iqf64(&rx_data);
        multiply_slice(&signal, &rx_signal, &mut tmp);
        let tmp_sum = sum_slice(&tmp);
        let mag = f64::sqrt(tmp_sum.re * tmp_sum.re + tmp_sum.im * tmp_sum.im);
        println!("mag:{:?}", mag);
    }

    rx_tx.send(true).expect("tried sending tx thread shutdown command");

    println!("joining tx thread");
    tx_handler.join().expect("joining tx thread");
}
