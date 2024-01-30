use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};
use whisper_rs_sys::{WHISPER_SAMPLE_RATE};
use std::path::PathBuf;
use symphonia::core::errors::Result as SymphoniaResult;
use symphonia::core::audio::{Signal, AudioBufferRef};
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::{MediaSourceStream, MediaSourceStreamOptions};
use symphonia::core::probe::Hint;
use symphonia::core::meta::MetadataOptions;
use std::fs::File;
use std::io::BufWriter;
use hound;

/// Transcribes the given audio file using the whisper-rs library.
///
/// # Arguments
///
/// * `file_path` - A string slice that holds the path to the audio file to be transcribed.
///
/// # Returns
///
/// A Result containing a String with the transcription if successful, or an error message if not.
pub fn transcribe(model_path: &PathBuf, file_path: &PathBuf) -> Result<String, String> {

    let model_path_str = model_path.to_str().expect("Not valid model path");
    // Load a context and model
    let ctx = WhisperContext::new_with_params(
        model_path_str, // Replace with the actual path to the model
        WhisperContextParameters::default(),
    )
    .map_err(|_| "failed to load model")?;

    // Create a state
    let mut state = ctx.create_state().map_err(|_| "failed to create state")?;

    // Create a params object
    // Note that currently the only implemented strategy is Greedy, BeamSearch is a WIP
    let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });

    // Edit parameters as needed
    params.set_n_threads(1); // Set the number of threads to use
    params.set_translate(true); // Enable translation
    params.set_language(Some("en")); // Set the language to translate to English
    // Disable printing to stdout
    params.set_print_special(false);
    params.set_print_progress(false);
    params.set_print_realtime(false);
    params.set_print_timestamps(false);

    // Load the audio file
    let audio_data = std::fs::read(file_path)//convert_webm_to_wav(file_path)
        .map_err(|e| format!("failed to read audio file: {}", e))?
        .chunks_exact(2)
        .map(|chunk| i16::from_ne_bytes([chunk[0], chunk[1]]))
        .collect::<Vec<i16>>();
    // Convert the audio data to the required format (16KHz mono i16 samples)
    let audio_data = whisper_rs::convert_integer_to_float_audio(&audio_data);
    // whisper_rs::convert_stereo_to_mono_audio(
    //     &whisper_rs::convert_integer_to_float_audio(&audio_data),
    // )
    // .map_err(|e| format!("failed to convert audio data: {}", e))?;

    // Run the model
    state.full(params, &audio_data[..]).map_err(|_| "failed to run model")?;

    // Fetch the results
    let num_segments = state.full_n_segments().map_err(|_| "failed to get number of segments")?;
    let mut transcription = String::new();
    for i in 0..num_segments {
        let segment = state.full_get_segment_text(i).map_err(|_| "failed to get segment")?;
        transcription.push_str(&segment);
        transcription.push('\n');
    }

    Ok(transcription)
}

fn convert_webm_to_wav(input_path: &PathBuf) -> SymphoniaResult<std::io::Cursor<Vec<u8>>> {
    // Open the input WebM file.
    let mss_options = MediaSourceStreamOptions::default();
    let mss = MediaSourceStream::new(Box::new(std::fs::File::open(input_path)?), mss_options);

    // Create a probe hint using the file's extension. [Optional]
    let mut hint = Hint::new();
    hint.mime_type("audio/webm");

    // Use the default options for metadata and format readers.
    let meta_opts: MetadataOptions = Default::default();
    let fmt_opts: FormatOptions = Default::default();

    // Probe the media source.
    let probed = symphonia::default::get_probe()
        .format(&hint, mss, &fmt_opts, &meta_opts)
        .expect("unsupported format");

    // Probe the input stream to determine the format.
    let mut format_reader = probed.format;

    // Get the default audio stream.
    let track = format_reader.default_track().unwrap();
    let track_id = track.id;

    // Create a decoder for the stream.
    // Use the default options for the decoder.
    let dec_opts: DecoderOptions = Default::default();

    println!("Code type: {}", track.codec_params.codec);

    // Create a decoder for the track.
    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &dec_opts)
        .expect("unsupported codec");

    // Prepare the output WAV file.
    let mut writer = std::io::Cursor::new(Vec::new());
    let spec = hound::WavSpec {
        channels: track.codec_params.channels.unwrap().count() as u16,
        sample_rate: WHISPER_SAMPLE_RATE,//track.codec_params.sample_rate.unwrap() as u32,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut wav_writer = hound::WavWriter::new(&mut writer, spec).map_err(|e| symphonia::core::errors::Error::DecodeError("Unable to create wav writer"))?;

    // Decode and write samples.
    while let packet = format_reader.next_packet()? {
        if packet.track_id() == track_id {
            // Decode the packet.
            let decoded = decoder.decode(&packet)?;

            // Get the buffer containing the audio samples.
            if let AudioBufferRef::F32(buffer) = decoded {
                // Convert the samples to PCM and write them to the WAV file.
                for sample in buffer.chan(0).iter() {
                    let pcm_sample = (sample * i16::MAX as f32) as i16;
                    wav_writer.write_sample(pcm_sample).map_err(|e| symphonia::core::errors::Error::DecodeError("Unable to write sample to wav"))?;
                }
            }
        }
    }

    // Finalize the WAV file.
    wav_writer.finalize().map_err(|e| symphonia::core::errors::Error::DecodeError("Unable to finalize wav"))?;

    // Save the in-memory WAV data to a file.
    let output_file_path = "output.wav";
    let mut output_file = std::fs::File::create(output_file_path)
        .expect("Unable to create output file");
    std::io::copy(&mut writer, &mut output_file)
        .expect("Unable to write WAV data to file");

    return Ok(writer);
}