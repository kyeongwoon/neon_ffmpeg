#![allow(non_snake_case)]
use neon::{prelude::*, types::buffer::TypedArray};
pub extern crate ffmpeg_sys as sys;
use std::ffi::*;
use sys::AVMediaType;

const INT64_MAX: i64 = 9223372036854775807;

fn av_dump_format(mut cx: FunctionContext) -> JsResult<JsObject> {
    let ic = cx.argument::<JsNumber>(0)?.value(&mut cx) as usize as *mut sys::AVFormatContext;
    let input_filename = cx.argument::<JsString>(1)?.value(&mut cx);
    unsafe { sys::av_dump_format(ic, 0, input_filename.as_ptr() as *const i8, 0) }

    let obj = cx.empty_object();

    let mut duration = 0 as i64;
    unsafe {
        if (*ic).duration != sys::AV_NOPTS_VALUE {
            duration = (*ic).duration
                + if (*ic).duration <= INT64_MAX - 5000 {
                    5000
                } else {
                    0
                };
            let mut secs = duration / sys::AV_TIME_BASE as i64;
            //let us = duration % sys::AV_TIME_BASE as i64;
            let mut mins = secs / 60;
            secs %= 60;
            let hours = mins / 60;
            mins %= 60;
        }
    }
    let duration = cx.number(duration as i32);
    obj.set(&mut cx, "duration", duration)?;

    unsafe {
        for i in 0..(*ic).nb_streams {
            let flags = (*(*ic).iformat).flags;
            let st = *(*ic).streams.offset(i as isize);
            let c_codec_name = sys::avcodec_get_name((*(*st).codecpar).codec_id);
            let c_codec_type = sys::av_get_media_type_string((*(*st).codecpar).codec_type);
            let codec_name = CStr::from_ptr(c_codec_name)
                .to_str()
                .expect("Failed to convert C string to Rust &str");

            let codec_type = CStr::from_ptr(c_codec_type)
                .to_str()
                .expect("Failed to convert C string to Rust &str");
            println!("{} {}", codec_name, codec_type);
        }
    }

    Ok(obj)
}

fn avformat_open_input(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let input_filename = cx.argument::<JsString>(0)?.value(&mut cx);
    let mut ic: *mut sys::AVFormatContext = std::ptr::null_mut();
    let iformat: *mut sys::AVInputFormat = std::ptr::null_mut();

    unsafe {
        sys::avformat_open_input(
            &mut ic as *mut *mut sys::AVFormatContext,
            input_filename.as_ptr() as *const c_char,
            iformat,
            std::ptr::null_mut(),
        );
    }
    Ok(cx.number(ic as usize as f64))
}

fn avformat_find_stream_info(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let ic = cx.argument::<JsNumber>(0)?.value(&mut cx) as usize as *mut sys::AVFormatContext;
    unsafe {
        sys::avformat_find_stream_info(ic, std::ptr::null_mut());
    }
    Ok(cx.undefined())
}

fn av_find_best_stream(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let ic = cx.argument::<JsNumber>(0)?.value(&mut cx) as usize as *mut sys::AVFormatContext;
    let v = cx.argument::<JsNumber>(1)?.value(&mut cx) as u32;
    let type_enum = match v {
        0 => AVMediaType::AVMEDIA_TYPE_VIDEO,
        1 => AVMediaType::AVMEDIA_TYPE_AUDIO,
        2 => AVMediaType::AVMEDIA_TYPE_DATA,
        3 => AVMediaType::AVMEDIA_TYPE_SUBTITLE,
        _ => AVMediaType::AVMEDIA_TYPE_UNKNOWN,
    };

    let mut stream =
        unsafe { sys::av_find_best_stream(ic, type_enum, -1, -1, std::ptr::null_mut(), 0) };
    if stream < 0 {
        stream = -1;
    }
    Ok(cx.number(stream as f64))
}

fn av_gettime(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let v = unsafe { sys::av_gettime() };
    Ok(cx.number(v as f64))
}

fn av_seek_frame(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let ic = cx.argument::<JsNumber>(0)?.value(&mut cx) as usize as *mut sys::AVFormatContext;
    let stream_index = cx.argument::<JsNumber>(1)?.value(&mut cx) as i32;
    let seek_target = cx.argument::<JsNumber>(2)?.value(&mut cx) as i64;
    let flags = cx.argument::<JsNumber>(3)?.value(&mut cx) as i32;

    let seek_target = unsafe {
        let av_stream = *(*ic).streams.offset(stream_index as isize);
        sys::av_rescale_q(seek_target, sys::AV_TIME_BASE_Q, (*av_stream).time_base)
    };

    let rc = unsafe { sys::av_seek_frame(ic, stream_index, seek_target, flags) };

    Ok(cx.number(rc as f64))
}

fn avformat_stream(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let ic = cx.argument::<JsNumber>(0)?.value(&mut cx) as usize as *mut sys::AVFormatContext;
    let num = cx.argument::<JsNumber>(1)?.value(&mut cx) as isize;

    let stream = unsafe { *(*ic).streams.offset(num) };
    Ok(cx.number(stream as usize as f64))
}

fn avformat_context(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let ic = cx.argument::<JsNumber>(0)?.value(&mut cx) as usize as *mut sys::AVFormatContext;
    let num = cx.argument::<JsNumber>(1)?.value(&mut cx) as isize;

    let codec_ctx = unsafe {
        let av_stream = *(*ic).streams.offset(num);

        if av_stream.is_null() {
            std::ptr::null_mut()
        } else {
            (*av_stream).codecpar
        }
    };
    Ok(cx.number(codec_ctx as usize as f64))
}

fn avformat_close_input(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let mut ic = cx.argument::<JsNumber>(0)?.value(&mut cx) as usize as *mut sys::AVFormatContext;
    unsafe { sys::avformat_close_input(&mut ic as *mut *mut sys::AVFormatContext) };
    Ok(cx.undefined())
}

fn avcodec_open(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let ic = cx.argument::<JsNumber>(0)?.value(&mut cx) as usize as *mut sys::AVFormatContext;
    let stream = cx.argument::<JsNumber>(1)?.value(&mut cx) as isize;

    let codec_par = unsafe { (*(*(*ic).streams.offset(stream))).codecpar };
    let codec = unsafe { sys::avcodec_find_decoder((*codec_par).codec_id) };
    if codec.is_null() {
        println!("codec is null");
    }
    let pCodecCtx = unsafe { sys::avcodec_alloc_context3(codec) };
    unsafe {
        let ret = sys::avcodec_parameters_to_context(
            pCodecCtx,
            (*(*(*ic).streams.offset(stream))).codecpar,
        );
        if ret != 0 {
            // error copying codec context
            print!("Could not copy codec context.\n");
        }
        if sys::avcodec_open2(pCodecCtx, codec, core::ptr::null_mut()) < 0 {
            println!("Could not open codec");
        }
    }

    Ok(cx.number(pCodecCtx as u64 as f64))
}

fn avcodec_close(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let codec_ctx = cx.argument::<JsNumber>(0)?.value(&mut cx) as usize as *mut sys::AVCodecContext;
    unsafe { sys::avcodec_close(codec_ctx) };
    Ok(cx.undefined())
}

const AV_CH_FRONT_LEFT: u64 = 1 << sys::AVChannel::AV_CHAN_FRONT_LEFT.0;
const AV_CH_FRONT_RIGHT: u64 = 1 << sys::AVChannel::AV_CHAN_FRONT_RIGHT.0;
const AV_CH_FRONT_CENTER: u64 = 1 << sys::AVChannel::AV_CHAN_FRONT_CENTER.0;

fn audio_resampling(
    audio_decode_ctx: *mut sys::AVCodecContext,
    decoded_audio_frame: *mut sys::AVFrame,
    out_sample_fmt: sys::AVSampleFormat,
    out_channels: i32,
    out_sample_rate: i32,
    out_buf: *mut u8,
) -> i32 {
    let mut resampled_data_size = 0;
    let mut swr_ctx = unsafe { sys::swr_alloc() };
    let mut out_channel_layout = AV_CH_FRONT_CENTER;
    //println!("out chanel_layout {:?}", out_channel_layout);

    let in_channel_layout = unsafe {
        let ctx_channels = (*audio_decode_ctx).channels;
        let layout_channels =
            sys::av_get_channel_layout_nb_channels((*audio_decode_ctx).channel_layout);
        if ctx_channels == layout_channels {
            ctx_channels
        } else {
            sys::av_get_default_channel_layout(ctx_channels) as i32
        }
    };
    if in_channel_layout < 0 {
        println!("in_channel_layout error.");
        return -1;
    }
    if out_channels == 1 {
        out_channel_layout = AV_CH_FRONT_CENTER;
    } else if out_channels == 2 {
        out_channel_layout = AV_CH_FRONT_LEFT | AV_CH_FRONT_RIGHT;
    } else {
        out_channel_layout = AV_CH_FRONT_LEFT | AV_CH_FRONT_RIGHT | AV_CH_FRONT_CENTER;
    }

    // retrieve number of audio samples (per channel)
    let mut in_nb_samples = unsafe { (*decoded_audio_frame).nb_samples };
    if in_nb_samples <= 0 {
        println!("in_nb_samples error.");
        return -1;
    }
    unsafe {
        // Set SwrContext parameters for resampling
        //  int av_opt_set_int(void *obj, const char *name, int64_t val, int search_flags)
        sys::av_opt_set_int(
            // 3
            swr_ctx as *mut c_void,
            CString::new("in_channel_count")
                .expect("CString::new failed")
                .as_ptr(),
            (*audio_decode_ctx).channels as i64,
            0,
        );
        sys::av_opt_set_int(
            // 3
            swr_ctx as *mut c_void,
            CString::new("in_channel_layout")
                .expect("CString::new failed")
                .as_ptr(),
            in_channel_layout as i64,
            0,
        );

        /*
        //20 int av_opt_get_int(void *obj, const char *name, int search_flags, int64_t *out_val)
        let mut val = 0 as i64;
        sys::av_opt_get_int(
            swr_ctx as *mut c_void,
            CString::new("in_channel_layout")
                .expect("CString::new failed")
                .as_ptr(),
            0,
            &mut val as *mut i64,
        );
        println!("in_channel_layout {} {}", in_channel_layout, val);
        */
        // Set SwrContext parameters for resampling
        sys::av_opt_set_int(
            swr_ctx as *mut c_void,
            CString::new("in_sample_rate")
                .expect("CString::new failed")
                .as_ptr(),
            (*audio_decode_ctx).sample_rate as i64,
            0,
        );

        // Set SwrContext parameters for resampling
        sys::av_opt_set_sample_fmt(
            swr_ctx as *mut c_void,
            CString::new("in_sample_fmt")
                .expect("CString::new failed")
                .as_ptr(),
            (*audio_decode_ctx).sample_fmt,
            0,
        );

        // Set SwrContext parameters for resampling
        sys::av_opt_set_int(
            swr_ctx as *mut c_void,
            CString::new("out_channel_count")
                .expect("CString::new failed")
                .as_ptr(),
            2 as i64,
            0,
        );
        sys::av_opt_set_int(
            swr_ctx as *mut c_void,
            CString::new("out_channel_layout")
                .expect("CString::new failed")
                .as_ptr(),
            out_channel_layout as i64,
            0,
        );

        // Set SwrContext parameters for resampling
        sys::av_opt_set_int(
            swr_ctx as *mut c_void,
            CString::new("out_sample_rate")
                .expect("CString::new failed")
                .as_ptr(),
            out_sample_rate as i64,
            0,
        );

        // Set SwrContext parameters for resampling
        sys::av_opt_set_sample_fmt(
            swr_ctx as *mut c_void,
            CString::new("out_sample_fmt")
                .expect("CString::new failed")
                .as_ptr(),
            out_sample_fmt,
            0,
        );
    }
    unsafe {
        sys::swr_init(swr_ctx);
    }

    // get number of output audio channels
    let mut out_nb_channels = unsafe { sys::av_get_channel_layout_nb_channels(out_channel_layout) };

    // retrieve output samples number taking into account the progressive delay
    // Rescale a 64-bit integer with specified rounding.  a * b / c
    unsafe {
        print!(
            "swr_get_delay {}\n",
            sys::swr_get_delay(swr_ctx, (*audio_decode_ctx).sample_rate as i64)
        );
    }

    let mut out_nb_samples = unsafe {
        sys::av_rescale_rnd(
            sys::swr_get_delay(swr_ctx, (*audio_decode_ctx).sample_rate as i64)
                + in_nb_samples as i64,
            out_sample_rate as i64,
            (*audio_decode_ctx).sample_rate as i64,
            sys::AVRounding::AV_ROUND_UP,
        )
    };

    //println!("out_nb_channels {} out_nb_samples {}", out_nb_channels, out_nb_samples);

    //")
    // check output samples number was correctly retrieved
    if out_nb_samples <= 0 {
        print!("av_rescale_rnd error\n");
        return -1;
    }
    //print!("in_nb_samples {}\n", in_nb_samples);
    //print!("out_nb_channels {}\n", out_nb_channels);
    //print!("out_nb_samples {}\n", out_nb_samples);
    //print!("out_sample_fmt {}\n", out_sample_fmt);

    let mut ret = 0;

    if swr_ctx.is_null() == false {
        let mut resampled_data: [*mut u8; 4] = [std::ptr::null_mut(); 4];
        let mut out_linesize = 0;
        resampled_data[0] = out_buf;
        //println!("resampled_data[0] {:?}", resampled_data[0]);
        //println!("resampled_data[1] {:?}", resampled_data[1]);

        // do the actual audio data resampling
        ret = unsafe {
            println!("frame->nb_samples {}", (*decoded_audio_frame).nb_samples);

            sys::swr_convert(
                swr_ctx,
                resampled_data.as_mut_ptr() as *mut *mut u8,
                out_nb_samples as i32,
                &mut (*decoded_audio_frame).data as *mut *mut u8 as *mut *const u8,
                (*decoded_audio_frame).nb_samples as i32,
            )
        };

        // check audio conversion was successful
        if ret < 0 {
            print!("swr_convert_error.\n");
            return -1;
        }

        // Get the required buffer size for the given audio parameters
        resampled_data_size = unsafe {
            sys::av_samples_get_buffer_size(
                &mut out_linesize,
                out_nb_channels,
                ret,
                out_sample_fmt,
                1,
            )
        };

        // check audio buffer size
        if resampled_data_size < 0 {
            print!("av_samples_get_buffer_size error.\n");
            return -1;
        }
    } else {
        print!("swr_ctx null error.\n");
        return -1;
    }

    unsafe { sys::swr_free(&mut swr_ctx as *mut *mut sys::SwrContext) };
    resampled_data_size
}

static mut SWR: *mut sys::SwrContext = std::ptr::null_mut();
static mut SWS: *mut sys::SwsContext = std::ptr::null_mut();
static mut VIDEO_CLOCK: f64 = 0.0;
static mut AUDIO_CLOCK: f64 = 0.0;

fn guess_correct_pts(ctx: *mut sys::AVCodecContext, reordered_pts: i64, dts: i64) -> f64 {
    let mut pts = sys::AV_NOPTS_VALUE as i64;
    unsafe {
        if dts != sys::AV_NOPTS_VALUE {
            if dts <= (*ctx).pts_correction_last_dts {
                (*ctx).pts_correction_num_faulty_dts += 1;
            }
            (*ctx).pts_correction_last_dts = dts;
        } else if reordered_pts != sys::AV_NOPTS_VALUE {
            (*ctx).pts_correction_last_dts = reordered_pts;
        }

        if reordered_pts != sys::AV_NOPTS_VALUE {
            if reordered_pts <= (*ctx).pts_correction_last_pts {
                (*ctx).pts_correction_num_faulty_pts += 1;
            }
            (*ctx).pts_correction_last_pts = reordered_pts;
        } else if dts != sys::AV_NOPTS_VALUE {
            (*ctx).pts_correction_last_pts = dts;
        }
        if ((*ctx).pts_correction_num_faulty_pts <= (*ctx).pts_correction_num_faulty_dts
            || dts == sys::AV_NOPTS_VALUE)
            || reordered_pts != sys::AV_NOPTS_VALUE
        {
            pts = reordered_pts;
        } else {
            pts = dts;
        }
    }
    pts as f64
}

fn avcodec_decode(mut cx: FunctionContext) -> JsResult<JsObject> {
    let ic = cx.argument::<JsNumber>(0)?.value(&mut cx) as usize as *mut sys::AVFormatContext;

    let vcodec_ctx = if cx.argument::<JsValue>(1)?.is_a::<JsNull, _>(&mut cx) {
        std::ptr::null_mut()
    } else {
        cx.argument::<JsNumber>(1)?.value(&mut cx) as usize as *mut sys::AVCodecContext
    };
    let vstream = cx.argument::<JsNumber>(2)?.value(&mut cx) as isize;

    let acodec_ctx = if cx.argument::<JsValue>(3)?.is_a::<JsNull, _>(&mut cx) {
        std::ptr::null_mut()
    } else {
        cx.argument::<JsNumber>(3)?.value(&mut cx) as usize as *mut sys::AVCodecContext
    };
    let astream = cx.argument::<JsNumber>(4)?.value(&mut cx) as isize;

    let scodec_ctx = if cx.argument::<JsValue>(5)?.is_a::<JsNull, _>(&mut cx) {
        std::ptr::null_mut()
    } else {
        cx.argument::<JsNumber>(5)?.value(&mut cx) as usize as *mut sys::AVCodecContext
    };
    let sstream = cx.argument::<JsNumber>(6)?.value(&mut cx) as isize;

    let buf = cx.argument::<JsTypedArray<u8>>(7)?;
    let size = buf.len(&mut cx);
    let data = buf.as_slice(&cx);
    //println!("size is {}", size);

    let data_ptr: *mut u8 = data.as_ptr() as *mut u8;

    let mut frame = unsafe { sys::av_frame_alloc() };
    let mut stream_type = -1;

    let mut packet: sys::AVPacket = unsafe { std::mem::zeroed() };
    //let mut packet: sys::AVPacket = unsafe { *sys::av_packet_alloc() };
    let mut pts = 0.0;
    let mut ret = 0;

    // for video
    let mut repeat_pict = 0;

    // for audio
    let mut audio_len = 0;
    let mut video_len = 0;
    let mut subtitle_len = 0;
    let mut sub_text = cx.string("");

    loop {
        ret = unsafe { sys::av_read_frame(ic, &mut packet) };
        if ret < 0 {
            println!("read frame error {}", ret);
            break;
        }
        //println!("packet.stream_index {}", packet.stream_index);
        if packet.stream_index == vstream as i32 {
            //println!("video");
            stream_type = 0;
            let mut got_frame = false;
            let mut ret = 0;
            ret = unsafe { sys::avcodec_send_packet(vcodec_ctx, &mut packet) };
            if ret < 0 {
                print!("Error sending packet for decoding.\n");
                break;
            }
            ret = unsafe { sys::avcodec_receive_frame(vcodec_ctx, frame) };
            if ret == sys::AVERROR(sys::EAGAIN) || ret == sys::AVERROR(sys::AVERROR_EOF) {
                break;
            } else if ret < 0 {
                print!("Error during decoding.\n");
                break;
            }
            let width = unsafe { (*vcodec_ctx).width };
            let height = unsafe { (*vcodec_ctx).height };
            let pix_fmt = unsafe { (*vcodec_ctx).pix_fmt };
            unsafe {
                SWS = sys::sws_getCachedContext(
                    SWS,
                    width,
                    height,
                    pix_fmt,
                    width,
                    height,
                    sys::AV_PIX_FMT_BGR32,
                    sys::SWS_FAST_BILINEAR,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                )
            };

            if packet.dts != sys::AV_NOPTS_VALUE {
                pts = unsafe { guess_correct_pts(vcodec_ctx, (*frame).pts, (*frame).pkt_dts) };
                if pts == sys::AV_NOPTS_VALUE as f64 {
                    pts = 0.0
                }
            } else {
                pts = 0.0
            }
            repeat_pict = unsafe { (*frame).repeat_pict };
            pts *= unsafe { sys::av_q2d((*(*(*ic).streams.offset(vstream as isize))).time_base) };
            {
                // sync video
                let mut frame_delay;
                if pts != 0.0 {
                    unsafe { VIDEO_CLOCK = pts };
                } else {
                    unsafe { pts = VIDEO_CLOCK };
                }
                frame_delay =
                    unsafe { sys::av_q2d((*(*(*ic).streams.offset(vstream as isize))).time_base) };
                // if we are repeating a frame, adjust clock accordingly
                frame_delay += repeat_pict as f64 * (frame_delay * 0.5);
                unsafe { VIDEO_CLOCK += frame_delay };

                unsafe {
                    let mut linesizes: [i32; 4] = [0; 4];
                    let mut pointers: [*mut u8; 4] = [std::ptr::null_mut(); 4];

                    linesizes[0] = width as i32 * 4;
                    linesizes[1] = 0;
                    linesizes[2] = 0;
                    linesizes[3] = 0;

                    pointers[0] = data_ptr;
                    pointers[1] = std::ptr::null_mut();
                    pointers[2] = std::ptr::null_mut();
                    pointers[3] = std::ptr::null_mut();

                    sys::sws_scale(
                        SWS,
                        (*frame).data.as_ptr() as *const *const u8,
                        (*frame).linesize.as_ptr() as *const i32,
                        0,
                        height,
                        &mut pointers as *const *mut u8,
                        &mut linesizes as *const i32,
                    );
                    video_len = linesizes[0] as i32 * height as i32;
                }
                //
            }
            break;
        } else if packet.stream_index == astream as i32 {
            let mut len1 = 0;
            let mut got_frame = false;
            //println!("audio");
            // Return decoded output data from a decoder or encoder
            let mut ret = unsafe { sys::avcodec_receive_frame(acodec_ctx, frame) };
            if ret == 0 {
                got_frame = true;
                //println!("got frame");
            }
            if ret == sys::AVERROR(sys::EAGAIN) {
                println!("EAGAIN while audio decoding.");
                ret = 0;
            }
            // Supply raw packet data as input to a decoder.
            if ret == 0 {
                ret = unsafe { sys::avcodec_send_packet(acodec_ctx, &mut packet) };
            }

            if ret == sys::AVERROR(sys::EAGAIN) {
                ret = 0;
            } else if ret < 0 {
                println!("avcodec_receive_frame error");
            } else {
                len1 = packet.size;
            }
            if len1 < 0 {
                // if error, skip frame
                println!("skip frame while audio decoding.\n");
            }

            if got_frame {
                stream_type = 1;
                if packet.pts != sys::AV_NOPTS_VALUE {
                    pts = packet.pts as f64;
                }
                pts *=
                    unsafe { sys::av_q2d((*(*(*ic).streams.offset(astream as isize))).time_base) };

                {
                    //let mut swr_ctx = std::ptr::null_mut();
                    let mut out_channel_layout = 0;
                    let mut out_nb_channels = 0;
                    let out_sample_rate = unsafe { (*acodec_ctx).sample_rate };
                    let out_sample_fmt = sys::AVSampleFormat::AV_SAMPLE_FMT_S16;

                    // retrieve number of audio samples (per channel)
                    let mut in_nb_samples = unsafe { (*frame).nb_samples };
                    if in_nb_samples <= 0 {
                        println!("in_nb_samples error.");
                    }
                    // get number of output audio channels
                    let mut out_nb_channels =
                        unsafe { sys::av_get_channel_layout_nb_channels(out_channel_layout) };

                    out_nb_channels = 2;

                    // retrieve output samples number taking into account the progressive delay
                    // Rescale a 64-bit integer with specified rounding.  a * b / c

                    let mut out_nb_samples = unsafe {
                        sys::av_rescale_rnd(
                            sys::swr_get_delay(SWR, (*acodec_ctx).sample_rate as i64)
                                + in_nb_samples as i64,
                            out_sample_rate as i64,
                            (*acodec_ctx).sample_rate as i64,
                            sys::AVRounding::AV_ROUND_UP,
                        )
                    };
                    if out_nb_samples <= 0 {
                        print!("av_rescale_rnd error\n");
                    }
                    //print!("in_nb_samples {}\n", in_nb_samples);
                    //print!("out_nb_channels {}\n", out_nb_channels);
                    //print!("out_nb_samples {}\n", out_nb_samples);

                    let mut resampled_data: [*mut u8; 4] = [std::ptr::null_mut(); 4];
                    let mut out_linesize = 0;
                    resampled_data[0] = data_ptr;
                    //println!("resampled_data[0] {:?}", resampled_data[0]);
                    //println!("resampled_data[1] {:?}", resampled_data[1]);

                    // do the actual audio data resampling
                    ret = unsafe {
                        //println!("frame->nb_samples {}", (*frame).nb_samples);

                        sys::swr_convert(
                            SWR,
                            resampled_data.as_mut_ptr() as *mut *mut u8,
                            out_nb_samples as i32,
                            &mut (*frame).data as *mut *mut u8 as *mut *const u8,
                            (*frame).nb_samples as i32,
                        )
                    };

                    // check audio conversion was successful
                    if ret < 0 {
                        print!("swr_convert_error.\n");
                        //return -1;
                    }

                    // Get the required buffer size for the given audio parameters
                    audio_len = unsafe {
                        sys::av_samples_get_buffer_size(
                            &mut out_linesize,
                            out_nb_channels,
                            ret,
                            out_sample_fmt,
                            1,
                        )
                    };
                    //println!("audio_len is {}", audio_len);
                }
                /*
                audio_len = unsafe {
                    audio_resampling(
                        acodec_ctx,
                        frame,
                        sys::AVSampleFormat::AV_SAMPLE_FMT_S16,
                        (*acodec_ctx).channels,
                        (*acodec_ctx).sample_rate,
                        data_ptr,
                    )
                };
                */
                //println!("audio_len is {}", audio_len);
            }
        } else if packet.stream_index == sstream as i32 {
            let mut got_subtitle = 0;
            let mut subtitle: sys::AVSubtitle = unsafe { std::mem::zeroed() };
/*
            let mut mysub: ::std::mem::MaybeUninit<sys::AVSubtitle> =
                ::std::mem::MaybeUninit::zeroed();
            //let mut mysub: ::std::mem::MaybeUninit<sys::AVSubtitle> = unsafe {::std::mem::MaybeUninit::uninit().assume_init()};
            println!("{:?}", mysub);
            let ptr = mysub.as_mut_ptr();
            unsafe { println!("{:?} {:?}", (*ptr).format, (*ptr).num_rects) };
*/
            let used = unsafe {
                sys::avcodec_decode_subtitle2(
                    scodec_ctx,
                    &mut subtitle,
                    //ptr,
                    &mut got_subtitle,
                    &mut packet,
                )
            };
            if used < 0 {
                println!("avcodec_decode_subtitle2 error.");
                break;
            }
            if got_subtitle == 1 {
                println!("got_subtitle");
                stream_type = 2;
                let num_of_rect = unsafe { subtitle.num_rects };
                for i in 0..num_of_rect {
                    let rect = unsafe { *subtitle.rects.offset(i as isize) };
                    let type_ = unsafe {(*rect).type_};
                    if type_ == sys::AVSubtitleType::SUBTITLE_TEXT {
                        let text = unsafe { (*rect).text };
                        let text = unsafe { std::ffi::CStr::from_ptr(text).to_str().unwrap() };
                        sub_text = cx.string(text);
                    } else if type_ == sys::AVSubtitleType::SUBTITLE_ASS {
                        let text = unsafe { (*rect).ass };
                        let text = unsafe { std::ffi::CStr::from_ptr(text).to_str().unwrap() };
                        sub_text = cx.string(text);
                    } else if type_ == sys::AVSubtitleType::SUBTITLE_BITMAP {
                        println!("SUBTITLE_BITMAP");
                    }
                }
                unsafe {
                    sys::avsubtitle_free(&mut subtitle);
                }
            }
        } else {
            println!("unknown stream {}", packet.stream_index);
        }
        break;
    }
    unsafe {
        //sys::av_packet_unref(&mut packet);
        sys::av_frame_free(&mut frame);
    }
    let obj = cx.empty_object();
    if ret == 0 {
        if stream_type == 0 {
            let val = cx.number(0);
            obj.set(&mut cx, "type", val)?;
            let val = cx.number(video_len);
            obj.set(&mut cx, "len", val)?;
            let val = cx.number(pts);
            obj.set(&mut cx, "pts", val)?;
            let val = cx.number(repeat_pict);
            obj.set(&mut cx, "repeat_pict", val)?;
        } else if stream_type == 1 {
            let val = cx.number(1);
            obj.set(&mut cx, "type", val)?;
            let val = cx.number(pts);
            obj.set(&mut cx, "pts", val)?;

            // 굳이 버퍼를 주지 말고, 길이만 알면된다.
            //let buf = JsBuffer::external(&mut cx, data_ptr);
            //obj.set(&mut cx, "data", buf)?;

            let val = cx.number(audio_len);
            obj.set(&mut cx, "len", val)?;
        } else if stream_type == 2 {
            let val = cx.number(2);
            obj.set(&mut cx, "type", val)?;
            obj.set(&mut cx, "text", sub_text)?;
        }
    } else {
        println!(
            "avcodec_decode error, ret is {}, stream_type is {}",
            ret, stream_type
        );
        let val = cx.number(ret);
        obj.set(&mut cx, "error", val)?;
    }
    Ok(obj)
}

fn avcodec_dimension(mut cx: FunctionContext) -> JsResult<JsArray> {
    let ic = cx.argument::<JsNumber>(0)?.value(&mut cx) as usize as *mut sys::AVFormatContext;
    let stream = cx.argument::<JsNumber>(1)?.value(&mut cx) as isize;

    let codec_param = unsafe { (*(*(*ic).streams.offset(stream))).codecpar };

    let w = unsafe { (*codec_param).width };
    let h = unsafe { (*codec_param).height };

    let w = cx.number(w);
    let h = cx.number(h);

    let arr = cx.empty_array();
    arr.set(&mut cx, 0, w)?;
    arr.set(&mut cx, 1, h)?;

    Ok(arr)
}

fn avcodec_resampler(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let audio_decode_ctx =
        cx.argument::<JsNumber>(0)?.value(&mut cx) as usize as *mut sys::AVCodecContext;

    //let mut swr_ctx = unsafe { sys::swr_alloc() };
    unsafe { SWR = sys::swr_alloc() };
    let mut out_channel_layout = AV_CH_FRONT_CENTER;
    let mut out_channels = 2;
    let mut out_sample_rate = 48000;
    let mut out_sample_fmt = sys::AVSampleFormat::AV_SAMPLE_FMT_S16;
    //println!("out chanel_layout {:?}", out_channel_layout);

    let in_channel_layout = unsafe {
        let ctx_channels = (*audio_decode_ctx).channels;
        let layout_channels =
            sys::av_get_channel_layout_nb_channels((*audio_decode_ctx).channel_layout);
        if ctx_channels == layout_channels {
            ctx_channels
        } else {
            sys::av_get_default_channel_layout(ctx_channels) as i32
        }
    };
    if in_channel_layout < 0 {
        println!("in_channel_layout error.");
        return Ok(cx.undefined());
    }

    let in_sample_rate = unsafe { (*audio_decode_ctx).sample_rate };
    let in_sample_fmt = unsafe { (*audio_decode_ctx).sample_fmt };
    if out_channels == 1 {
        out_channel_layout = AV_CH_FRONT_CENTER;
    } else if out_channels == 2 {
        out_channel_layout = AV_CH_FRONT_LEFT | AV_CH_FRONT_RIGHT;
    } else {
        out_channel_layout = AV_CH_FRONT_LEFT | AV_CH_FRONT_RIGHT | AV_CH_FRONT_CENTER;
    }

    // retrieve number of audio samples (per channel)
    unsafe {
        // Set SwrContext parameters for resampling
        //  int av_opt_set_int(void *obj, const char *name, int64_t val, int search_flags)
        sys::av_opt_set_int(
            // 3
            SWR as *mut c_void,
            CString::new("in_channel_count")
                .expect("CString::new failed")
                .as_ptr(),
            (*audio_decode_ctx).channels as i64,
            0,
        );
        sys::av_opt_set_int(
            // 3
            SWR as *mut c_void,
            CString::new("in_channel_layout")
                .expect("CString::new failed")
                .as_ptr(),
            in_channel_layout as i64,
            0,
        );

        // Set SwrContext parameters for resampling
        sys::av_opt_set_int(
            SWR as *mut c_void,
            CString::new("in_sample_rate")
                .expect("CString::new failed")
                .as_ptr(),
            (*audio_decode_ctx).sample_rate as i64,
            0,
        );

        // Set SwrContext parameters for resampling
        sys::av_opt_set_sample_fmt(
            SWR as *mut c_void,
            CString::new("in_sample_fmt")
                .expect("CString::new failed")
                .as_ptr(),
            (*audio_decode_ctx).sample_fmt,
            0,
        );

        // Set SwrContext parameters for resampling
        sys::av_opt_set_int(
            SWR as *mut c_void,
            CString::new("out_channel_count")
                .expect("CString::new failed")
                .as_ptr(),
            2 as i64,
            0,
        );
        sys::av_opt_set_int(
            SWR as *mut c_void,
            CString::new("out_channel_layout")
                .expect("CString::new failed")
                .as_ptr(),
            out_channel_layout as i64,
            0,
        );

        // Set SwrContext parameters for resampling
        sys::av_opt_set_int(
            SWR as *mut c_void,
            CString::new("out_sample_rate")
                .expect("CString::new failed")
                .as_ptr(),
            out_sample_rate as i64,
            0,
        );

        // Set SwrContext parameters for resampling
        sys::av_opt_set_sample_fmt(
            SWR as *mut c_void,
            CString::new("out_sample_fmt")
                .expect("CString::new failed")
                .as_ptr(),
            out_sample_fmt,
            0,
        );
    }
    unsafe {
        sys::swr_init(SWR);
    }

    Ok(cx.undefined())
}

fn avcodec_sample_rate(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let codec_ctx = cx.argument::<JsNumber>(0)?.value(&mut cx) as usize as *mut sys::AVCodecContext;

    let sample_rate = unsafe { (*codec_ctx).sample_rate };
    Ok(cx.number(sample_rate as f64))
}

fn avcodec_timebase(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let codec_ctx = cx.argument::<JsNumber>(0)?.value(&mut cx) as usize as *mut sys::AVCodecContext;
    let time_base = unsafe { sys::av_q2d((*codec_ctx).time_base) };

    Ok(cx.number(time_base))
}

fn av_usleep(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let v = cx.argument::<JsNumber>(0)?.value(&mut cx) as u32;
    unsafe {
        sys::av_usleep(v);
    }
    Ok(cx.undefined())
}

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    let val = cx.number(AVMediaType::AVMEDIA_TYPE_VIDEO as i32 as f64);
    cx.export_value("AVMEDIA_TYPE_VIDEO", val)?;
    let val = cx.number(AVMediaType::AVMEDIA_TYPE_AUDIO as i32 as f64);
    cx.export_value("AVMEDIA_TYPE_AUDIO", val)?;
    let val = cx.number(AVMediaType::AVMEDIA_TYPE_SUBTITLE as i32 as f64);
    cx.export_value("AVMEDIA_TYPE_SUBTITLE", val)?;

    let val = cx.number(sys::AVSEEK_FLAG_BACKWARD);
    cx.export_value("AVSEEK_FLAG_BACKWARD", val)?;

    cx.export_function("avformat_open_input", avformat_open_input)?;
    cx.export_function("avformat_find_stream_info", avformat_find_stream_info)?;
    cx.export_function("av_dump_format", av_dump_format)?;
    cx.export_function("av_find_best_stream", av_find_best_stream)?;
    cx.export_function("av_gettime", av_gettime)?;
    cx.export_function("av_seek_frame", av_seek_frame)?;
    cx.export_function("avformat_stream", avformat_stream)?;
    cx.export_function("avformat_context", avformat_context)?;
    cx.export_function("avformat_close_input", avformat_close_input)?;

    cx.export_function("avcodec_open", avcodec_open)?;
    cx.export_function("avcodec_close", avcodec_close)?;
    cx.export_function("avcodec_decode", avcodec_decode)?;
    cx.export_function("avcodec_dimension", avcodec_dimension)?;
    cx.export_function("avcodec_resampler", avcodec_resampler)?;
    cx.export_function("avcodec_sample_rate", avcodec_sample_rate)?;
    cx.export_function("avcodec_timebase", avcodec_timebase)?;
    cx.export_function("av_usleep", av_usleep)?;

    Ok(())
}
