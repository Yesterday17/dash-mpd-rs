// Tests for MPD download/transcoding support
//
// To run tests while enabling printing to stdout/stderr
//
//    cargo test --test transcoding -- --show-output


use fs_err as fs;
use std::env;
use std::path::PathBuf;
use ffprobe::ffprobe;
use file_format::FileFormat;
use dash_mpd::fetch::DashDownloader;


// We tolerate significant differences in final output file size, because as encoder performance
// changes in newer versions of ffmpeg, the resulting file size when reencoding may change
// significantly.
fn check_file_size_approx(p: &PathBuf, expected: u64) {
    let meta = fs::metadata(p).unwrap();
    let ratio = meta.len() as f64 / expected as f64;
    assert!(0.9 < ratio && ratio < 1.1, "File sizes: expected {}, got {}", expected, meta.len());
}


// We can't check file size for this test, as depending on whether mkvmerge or ffmpeg or mp4box are
// used to copy the video stream into the Matroska container (depending on which one is installed),
// the output file size varies quite a lot.
#[tokio::test]
#[cfg(not(feature = "libav"))]
async fn test_dl_mkv() {
    let mpd_url = "https://cloudflarestream.com/31c9291ab41fac05471db4e73aa11717/manifest/video.mpd";
    let out = env::temp_dir().join("cf.mkv");
    DashDownloader::new(mpd_url)
        .worst_quality()
        .verbosity(3)
        .download_to(out.clone()).await
        .unwrap();
    let format = FileFormat::from_file(out.clone()).unwrap();
    assert_eq!(format, FileFormat::MatroskaVideo);
    println!("DASH content saved to MKV container at {}", out.to_string_lossy());
}

#[tokio::test]
#[cfg(not(feature = "libav"))]
async fn test_dl_webm() {
    let mpd_url = "https://cloudflarestream.com/31c9291ab41fac05471db4e73aa11717/manifest/video.mpd";
    let out = env::temp_dir().join("cf.webm");
    DashDownloader::new(mpd_url)
        .worst_quality()
        .download_to(out.clone()).await
        .unwrap();
    check_file_size_approx(&out, 65_798);
    let format = FileFormat::from_file(out.clone()).unwrap();
    assert_eq!(format, FileFormat::Webm);
}

#[tokio::test]
#[cfg(not(feature = "libav"))]
async fn test_dl_avi() {
    if env::var("CI").is_ok() {
        return;
    }
    let mpd_url = "https://m.dtv.fi/dash/dasherh264/manifest.mpd";
    let out = env::temp_dir().join("caminandes.avi");
    DashDownloader::new(mpd_url)
        .worst_quality()
        .download_to(out.clone()).await
        .unwrap();
    check_file_size_approx(&out, 7_128_748);
    let format = FileFormat::from_file(out.clone()).unwrap();
    assert_eq!(format, FileFormat::AudioVideoInterleave);
}

#[tokio::test]
#[cfg(not(feature = "libav"))]
async fn test_dl_av1() {
    if env::var("CI").is_ok() {
        return;
    }
    // from demo page at https://bitmovin.com/demos/av1
    let mpd_url = "https://storage.googleapis.com/bitmovin-demos/av1/stream.mpd";
    let out = env::temp_dir().join("mango.webm");
    DashDownloader::new(mpd_url)
        .worst_quality()
        .download_to(out.clone()).await
        .unwrap();
    check_file_size_approx(&out, 12_987_188);
    let meta = ffprobe(out.clone()).unwrap();
    assert_eq!(meta.streams.len(), 2);
    let video = &meta.streams[0];
    assert_eq!(video.codec_type, Some(String::from("video")));
    assert_eq!(video.codec_name, Some(String::from("av1")));
    assert!(video.width.is_some());
    let audio = &meta.streams[1];
    assert_eq!(audio.codec_type, Some(String::from("audio")));
    assert_eq!(audio.codec_name, Some(String::from("opus")));
}


// Test transcoding audio from mp4a/aac to Ogg Vorbis
#[tokio::test]
#[cfg(not(feature = "libav"))]
async fn test_dl_audio_vorbis() {
    if env::var("CI").is_ok() {
        return;
    }
    let mpd_url = "https://dash.akamaized.net/dash264/TestCases/3a/fraunhofer/aac-lc_stereo_without_video/Sintel/sintel_audio_only_aaclc_stereo_sidx.mpd";
    let out = env::temp_dir().join("sintel-audio.ogg");
    DashDownloader::new(mpd_url)
        .worst_quality()
        .download_to(out.clone()).await
        .unwrap();
    check_file_size_approx(&out, 9_880_500);
    let format = FileFormat::from_file(out.clone()).unwrap();
    assert_eq!(format, FileFormat::OggVorbis);
    let meta = ffprobe(out.clone()).unwrap();
    assert_eq!(meta.streams.len(), 1);
    let audio = &meta.streams[0];
    assert_eq!(audio.codec_type, Some(String::from("audio")));
    assert_eq!(audio.codec_name, Some(String::from("vorbis")));
}

// Test transcoding multiperiod audio from mp4a/aac to MP3
#[tokio::test]
#[cfg(not(feature = "libav"))]
async fn test_dl_audio_multiperiod_mp3() {
    if env::var("CI").is_ok() {
        return;
    }
    let mpd_url = "https://media.axprod.net/TestVectors/v7-Clear/Manifest_MultiPeriod_AudioOnly.mpd";
    let out = env::temp_dir().join("multiperiod-audio.mp3");
    DashDownloader::new(mpd_url)
        .worst_quality()
        .download_to(out.clone()).await
        .unwrap();
    check_file_size_approx(&out, 23_362_703);
    let format = FileFormat::from_file(out.clone()).unwrap();
    assert_eq!(format, FileFormat::Mpeg12AudioLayer3);
    let meta = ffprobe(out.clone()).unwrap();
    assert_eq!(meta.streams.len(), 1);
    let audio = &meta.streams[0];
    assert_eq!(audio.codec_type, Some(String::from("audio")));
    assert_eq!(audio.codec_name, Some(String::from("mp3")));
}


