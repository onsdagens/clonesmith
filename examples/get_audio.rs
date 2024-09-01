use std::sync::Mutex;
use std::sync::Arc;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::io::BufWriter;
use std::fs::File;
use hound;
use cpal::{FromSample, Sample};

fn main() {
    println!("Available hosts:");
    for (i, host) in cpal::available_hosts().iter().enumerate() {
        println!("[{}]{:?}", i, host);
    }
    println!("Pick host: ");

    let choice = get_choice();
    println!("Chosen host:{:?}", cpal::available_hosts()[choice]);
    let host = cpal::host_from_id(cpal::available_hosts()[choice]).unwrap();
    let devices: &mut Vec<_> = &mut host.input_devices().unwrap().collect();
    println!("Available devices:");
    let mut i = 0;
    for device in devices.clone() {
        println!("[{}]{}",i, device.name().unwrap_or("".to_string()));
        i += 1;
    }
    println!("Pick device:");
    let choice = get_choice();
    let device = &devices[choice];
    println!("Picked {}", device.name().unwrap());
   
    let default_config = device.default_output_config().unwrap();
    println!("Default config: {:?}", default_config);


    let err_fn = move |err| {
        eprintln!("an error occurred on stream: {}", err);
    };
    const PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/recorded.wav");
    let spec = wav_spec_from_config(&default_config);
    let writer = hound::WavWriter::create(PATH, spec).unwrap();
    let writer = Arc::new(Mutex::new(Some(writer)));
    let writer_2 = writer.clone();
    let stream = device.build_input_stream(
            &default_config.into(),
            move |data, _: &_| write_input_data::<f32, f32>(data, &writer_2),
            err_fn,
            None,
        ).unwrap();
    stream.play().ok();

    // Let recording go for roughly three seconds.
    std::thread::sleep(std::time::Duration::from_secs(3));
    drop(stream);
    writer.lock().unwrap().take().unwrap().finalize().unwrap();
    println!("Recording {} complete!", PATH);
}

fn wav_spec_from_config(config: &cpal::SupportedStreamConfig) -> hound::WavSpec {
    hound::WavSpec {
        channels: config.channels() as _,
        sample_rate: config.sample_rate().0 as _,
        bits_per_sample: (config.sample_format().sample_size() * 8) as _,
        sample_format: sample_format(config.sample_format()),
    }
}
type WavWriterHandle = Arc<Mutex<Option<hound::WavWriter<BufWriter<File>>>>>;

fn write_input_data<T, U>(input: &[T], writer: &WavWriterHandle)
where
    T: Sample,
    U: Sample + hound::Sample + FromSample<T>,
{
    if let Ok(mut guard) = writer.try_lock() {
        if let Some(writer) = guard.as_mut() {
            for &sample in input.iter() {
                let sample: U = U::from_sample(sample);
                writer.write_sample(sample).ok();
            }
        }
    }
}

fn get_choice() -> usize {
    use std::io::{stdin, stdout, Write};
    let mut choice = String::new();
    let _ = stdout().flush();
    stdin().read_line(&mut choice).ok();
    let choice = choice.trim();
    choice.parse::<usize>().unwrap()
}

fn sample_format(format: cpal::SampleFormat) -> hound::SampleFormat {
    if format.is_float() {
        hound::SampleFormat::Float
    } else {
        hound::SampleFormat::Int
    }
}
