use mp4::{
    AudioObjectType, AvcProfile, ChannelConfig, MediaType, Metadata, Mp4Reader, SampleFreqIndex,
    TrackType,
};
use std::fs::{self, File};
use std::io::BufReader;
use std::time::Duration;

#[test]
fn test_read_mp4() {
    let mut mp4 = get_reader("tests/samples/minimal.mp4");

    assert_eq!(2591, mp4.size());

    // ftyp.
    assert_eq!(4, mp4.compatible_brands().len());

    // Check compatible_brands.
    let brands = vec![
        String::from("isom"),
        String::from("iso2"),
        String::from("avc1"),
        String::from("mp41"),
    ];

    for b in brands {
        let t = mp4.compatible_brands().iter().any(|x| x.to_string() == b);
        assert!(t);
    }

    assert_eq!(mp4.duration(), Duration::from_millis(62));
    assert_eq!(mp4.timescale(), 1000);
    assert_eq!(mp4.tracks().len(), 2);

    let sample_count = mp4.sample_count(1).unwrap();
    assert_eq!(sample_count, 1);
    let sample_1_1 = mp4.read_sample(1, 1).unwrap().unwrap();
    assert_eq!(sample_1_1.bytes.len(), 751);
    assert_eq!(
        sample_1_1,
        mp4::Mp4Sample {
            start_time: 0,
            duration: 512,
            rendering_offset: 0,
            is_sync: true,
            bytes: vec![0x0u8; 751],
        }
    );
    let eos = mp4.read_sample(1, 2).unwrap();
    assert!(eos.is_none());

    let sample_count = mp4.sample_count(2).unwrap();
    assert_eq!(sample_count, 3);
    let sample_2_1 = mp4.read_sample(2, 1).unwrap().unwrap();
    assert_eq!(sample_2_1.bytes.len(), 179);
    assert_eq!(
        sample_2_1,
        mp4::Mp4Sample {
            start_time: 0,
            duration: 1024,
            rendering_offset: 0,
            is_sync: true,
            bytes: vec![0x0u8; 179],
        }
    );

    let sample_2_2 = mp4.read_sample(2, 2).unwrap().unwrap();
    assert_eq!(
        sample_2_2,
        mp4::Mp4Sample {
            start_time: 1024,
            duration: 1024,
            rendering_offset: 0,
            is_sync: true,
            bytes: vec![0x0u8; 180],
        }
    );

    let sample_2_3 = mp4.read_sample(2, 3).unwrap().unwrap();
    assert_eq!(
        sample_2_3,
        mp4::Mp4Sample {
            start_time: 2048,
            duration: 896,
            rendering_offset: 0,
            is_sync: true,
            bytes: vec![0x0u8; 160],
        }
    );

    let eos = mp4.read_sample(2, 4).unwrap();
    assert!(eos.is_none());

    // track #1
    let track1 = mp4.tracks().get(&1).unwrap();
    assert_eq!(track1.track_id(), 1);
    assert_eq!(track1.track_type().unwrap(), TrackType::Video);
    assert_eq!(track1.media_type().unwrap(), MediaType::H264);
    assert_eq!(track1.video_profile().unwrap(), AvcProfile::AvcHigh);
    assert_eq!(track1.width(), 320);
    assert_eq!(track1.height(), 240);
    assert_eq!(track1.bitrate(), 150200);
    assert_eq!(track1.frame_rate(), 25.00);

    // track #2
    let track2 = mp4.tracks().get(&2).unwrap();
    assert_eq!(track2.track_type().unwrap(), TrackType::Audio);
    assert_eq!(track2.media_type().unwrap(), MediaType::AAC);
    assert_eq!(
        track2.audio_profile().unwrap(),
        AudioObjectType::AacLowComplexity
    );
    assert_eq!(
        track2.sample_freq_index().unwrap(),
        SampleFreqIndex::Freq48000
    );
    assert_eq!(track2.channel_config().unwrap(), ChannelConfig::Mono);
    assert_eq!(track2.bitrate(), 67695);
}

#[test]
fn test_read_extended_audio_object_type() {
    // Extended audio object type and sample rate index of 15
    let mp4 = get_reader("tests/samples/extended_audio_object_type.mp4");

    let track = mp4.tracks().get(&1).unwrap();
    assert_eq!(track.track_type().unwrap(), TrackType::Audio);
    assert_eq!(track.media_type().unwrap(), MediaType::AAC);
    assert_eq!(
        track.audio_profile().unwrap(),
        AudioObjectType::AudioLosslessCoding
    );
    assert_eq!(
        track
            .trak
            .mdia
            .minf
            .stbl
            .stsd
            .mp4a
            .as_ref()
            .unwrap()
            .esds
            .as_ref()
            .unwrap()
            .es_desc
            .dec_config
            .dec_specific
            .freq_index,
        15
    );
    assert_eq!(track.channel_config().unwrap(), ChannelConfig::Stereo);
    assert_eq!(track.bitrate(), 839250);
}

fn get_reader(path: &str) -> Mp4Reader<BufReader<File>> {
    let f = File::open(path).unwrap();
    let f_size = f.metadata().unwrap().len();
    let reader = BufReader::new(f);

    mp4::Mp4Reader::read_header(reader, f_size).unwrap()
}

#[test]
fn test_read_metadata() {
    let want_poster = fs::read("tests/samples/big_buck_bunny.jpg").unwrap();
    let want_summary = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed non risus. Suspendisse lectus tortor, dignissim sit amet, adipiscing nec, ultricies sed, dolor. Cras elementum ultrices diam. Maecenas ligula massa, varius a, semper congue, euismod non, mi. Proin porttitor, orci nec nonummy molestie, enim est eleifend mi, non fermentum diam nisl sit amet erat. Duis semper. Duis arcu massa, scelerisque vitae, consequat in, pretium a, enim. Pellentesque congue. Ut in risus volutpat libero pharetra tempor. Cras vestibulum bibendum augue.";
    let mp4 = get_reader("tests/samples/big_buck_bunny_metadata.m4v");
    let metadata = mp4.metadata();
    assert_eq!(metadata.title(), Some("Big Buck Bunny".into()));
    assert_eq!(metadata.year(), Some(2008));
    assert_eq!(metadata.summary(), Some(want_summary.into()));

    assert!(metadata.poster().is_some());
    let poster = metadata.poster().unwrap();
    assert_eq!(poster.len(), want_poster.len());
    assert_eq!(poster, want_poster.as_slice());
}

#[test]
fn test_read_fragments() {
    let mp4 = get_reader("tests/samples/minimal_init.mp4");

    assert_eq!(692, mp4.size());
    assert_eq!(5, mp4.compatible_brands().len());

    let sample_count = mp4.sample_count(1).unwrap();
    assert_eq!(sample_count, 0);

    let f = File::open("tests/samples/minimal_fragment.m4s").unwrap();
    let f_size = f.metadata().unwrap().len();
    let frag_reader = BufReader::new(f);

    let mut mp4_fragment = mp4.read_fragment_header(frag_reader, f_size).unwrap();
    let sample_count = mp4_fragment.sample_count(1).unwrap();
    assert_eq!(sample_count, 1);
    let sample_1_1 = mp4_fragment.read_sample(1, 1).unwrap().unwrap();
    assert_eq!(sample_1_1.bytes.len(), 751);
    assert_eq!(
        sample_1_1,
        mp4::Mp4Sample {
            start_time: 0,
            duration: 512,
            rendering_offset: 0,
            is_sync: true,
            bytes: vec![0x0u8; 751],
        }
    );
    let eos = mp4_fragment.read_sample(1, 2);
    assert!(eos.is_err());
}
