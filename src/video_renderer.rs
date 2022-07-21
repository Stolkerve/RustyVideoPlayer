#![allow(unused_assignments)]

pub struct VideoContext {
    pub video_width: i32, pub video_height: i32,

    pub time_base: ffmpeg_sys_next::AVRational,
    pub sws_scale_ctx: *mut ffmpeg_sys_next::SwsContext,
    pub format_context: *mut ffmpeg_sys_next::AVFormatContext,
    pub video_codec_parameters: *mut ffmpeg_sys_next::AVCodecParameters,
    pub video_codec: *mut ffmpeg_sys_next::AVCodec,
    pub video_stream_index: i32,
    pub codec_context: *mut ffmpeg_sys_next::AVCodecContext,
    pub frame: *mut ffmpeg_sys_next::AVFrame,
    pub packet: *mut ffmpeg_sys_next::AVPacket
}

pub unsafe fn load_video(video_ctx: &mut VideoContext, video_path: &str) {
    let mut format_context = ffmpeg_sys_next::avformat_alloc_context();
    assert!(!format_context.is_null(), "ERROR could not allocate memory for Format Context");

    let cstr_video_path = std::ffi::CString::new(video_path).unwrap();
    if ffmpeg_sys_next::avformat_open_input(&mut format_context, cstr_video_path.as_ptr(), std::ptr::null_mut(), std::ptr::null_mut()) != 0 {
        assert!(false, "ERROR could not open the file");
    }
    println!("Format {}, duration {} us", std::ffi::CStr::from_ptr((*(*format_context).iformat).long_name).to_str().unwrap(), (*format_context).duration);

    let mut video_codec_parameters: *mut ffmpeg_sys_next::AVCodecParameters = std::ptr::null_mut();
    let mut video_codec: *mut ffmpeg_sys_next::AVCodec = std::ptr::null_mut();
    let mut video_stream_index: i32 = -1;

    for i in 0..(*format_context).nb_streams {
        let local_codec_parameters = (*(*(*format_context).streams).offset(i as isize)).codecpar;
        let local_codec = ffmpeg_sys_next::avcodec_find_decoder((*local_codec_parameters).codec_id); if local_codec.is_null() {
            println!("ERROR unsupported codec!");
            continue;
        }

        if (*local_codec_parameters).codec_type == ffmpeg_sys_next::AVMediaType::AVMEDIA_TYPE_VIDEO {
            video_stream_index = i as i32;
            video_codec_parameters = local_codec_parameters;
            video_codec = local_codec;
            (*video_ctx).time_base = (*(*(*format_context).streams).offset(i as isize)).time_base;
            (*video_ctx).video_width = (*video_codec_parameters).width;
            (*video_ctx).video_height = (*video_codec_parameters).height;

            println!("Video resolution: {} x {}", (*local_codec_parameters).width, (*local_codec_parameters).height);
            break; 
        }
        else if (*local_codec_parameters).codec_type == ffmpeg_sys_next::AVMediaType::AVMEDIA_TYPE_AUDIO {
            println!("Audio channels: {}, sample rate: {}", (*local_codec_parameters).channels, (*local_codec_parameters).sample_rate); 
        }
        println!("\tID: {:?}, bitrate: {}", (*local_codec).id, (*local_codec_parameters).bit_rate);
    }

    assert!(video_stream_index != -1, "File does not contain a video stream!");

    let codec_context = ffmpeg_sys_next::avcodec_alloc_context3(video_codec);
    assert!(!codec_context.is_null(), "Failed to allocate memory for AVCodecContext");

    if ffmpeg_sys_next::avcodec_parameters_to_context(codec_context, video_codec_parameters) < 0 {
        assert!(false, "failed to copy codec params to codec context");
    }

    if ffmpeg_sys_next::avcodec_open2(codec_context, video_codec, std::ptr::null_mut()) < 0 {
        assert!(false, "failed to open codec through avcodec_open2");
    }

    let frame = ffmpeg_sys_next::av_frame_alloc();
    assert!(!frame.is_null(), "failed to allocate memory for AVFrame");
    let packet = ffmpeg_sys_next::av_packet_alloc();
    assert!(!packet.is_null(), "failed to allocate memory for AVPacket");

    (*video_ctx).format_context = format_context;
    (*video_ctx).video_codec_parameters =  video_codec_parameters;
    (*video_ctx).video_codec = video_codec;
    (*video_ctx).video_stream_index = video_stream_index;
    (*video_ctx).codec_context = codec_context;
    (*video_ctx).frame = frame;
    (*video_ctx).packet = packet;
}

pub unsafe fn read_video_frame(video_ctx: &mut VideoContext, data: &mut Vec<u8>, pts: &mut i64) {
    let format_context = video_ctx.format_context;
    let codec_context = video_ctx.codec_context;
    let video_stream_index = video_ctx.video_stream_index;

    let packet = video_ctx.packet;
    let frame = video_ctx.frame;

    let mut response = 0;
    while ffmpeg_sys_next::av_read_frame(format_context, packet) >= 0 {
        if (*packet).stream_index != video_stream_index {
            continue;
        }

        response = ffmpeg_sys_next::avcodec_send_packet(codec_context, packet);
        if response < 0 {
            println!("Failed to decode packet");
            break;
        }

        response = ffmpeg_sys_next::avcodec_receive_frame(codec_context, frame);
        if (response == ffmpeg_sys_next::AVERROR(ffmpeg_sys_next::EAGAIN)) || (response == ffmpeg_sys_next::AVERROR_EOF) {
            continue;
        }
        else if response < 0 {
            println!("Failed to decode packet");
            break;
        }

        ffmpeg_sys_next::av_packet_unref(packet);
        break;
    }
    *pts = (*frame).pts;

    data.reserve_exact(((*frame).width * (*frame).height * 4) as usize);

    if (*video_ctx).sws_scale_ctx.is_null() {
        (*video_ctx).sws_scale_ctx = ffmpeg_sys_next::sws_getContext(
            (*frame).width,
            (*frame).height,
            (*codec_context).pix_fmt,
            (*frame).width,
            (*frame).height,
            ffmpeg_sys_next::AVPixelFormat::AV_PIX_FMT_RGB0,
            ffmpeg_sys_next::SWS_BILINEAR,
            std::ptr::null_mut(), std::ptr::null_mut(), std::ptr::null_mut()
        );
    }

    assert!(!(*video_ctx).sws_scale_ctx.is_null(), "Couldn't initialize sw scaler");

    let dest = [data.as_mut_ptr(), std::ptr::null_mut(), std::ptr::null_mut(), std::ptr::null_mut()];
    let dest_linesize = [(*frame).width * 4, 0, 0, 0]; 
    ffmpeg_sys_next::sws_scale(
        (*video_ctx).sws_scale_ctx,
        (*frame).data.as_ptr() as *const *const u8,
        (*frame).linesize.as_ptr(),
        0,
        (*frame).height,
        dest.as_ptr(),
        dest_linesize.as_ptr()
    );
}

pub unsafe fn free_video_data(video_ctx: &mut VideoContext) {
    ffmpeg_sys_next::sws_freeContext(video_ctx.sws_scale_ctx);
    ffmpeg_sys_next::avformat_close_input(&mut video_ctx.format_context);
    ffmpeg_sys_next::avformat_free_context(video_ctx.format_context);
    ffmpeg_sys_next::avcodec_free_context(&mut video_ctx.codec_context);
    ffmpeg_sys_next::av_free_packet(video_ctx.packet);
    ffmpeg_sys_next::av_frame_free(&mut video_ctx.frame);
}
