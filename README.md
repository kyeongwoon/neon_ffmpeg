# neon_ffmpeg
This library was written on top of Rust FFI bindings for ffmpeg sys. [ffmpeg sys](https://github.com/meh/rust-ffmpeg/tree/master/sys) 


This project was bootstrapped by [create-neon](https://www.npmjs.com/package/create-neon).

## Installing neon_ffmpeg

Installing neon_al requires a [supported version of Node and Rust](https://github.com/neon-bindings/neon#platform-support).

You can install the project with npm. In the project directory, run:

```sh
$ npm install
```

This fully installs the project, including installing any dependencies and running the build.

## Building neon_ffmpeg

If you have already installed the project and only want to run the build, run:

```sh
$ npm run build
```

This command uses the [cargo-cp-artifact](https://github.com/neon-bindings/cargo-cp-artifact) utility to run the Rust build and copy the built library into `./index.node`.

## Usage

**Open media file using ffmpeg**
```javascript
import { createRequire } from 'module';
const AV = createRequire(import.meta.url)('./neon_ffmpeg/index.node')

const ic = AV.avformat_open_input(path);
//AV.av_dump_format(ic, path)
AV.avformat_find_stream_info(ic);

const audioStream = AV.av_find_best_stream(ic, AV.AVMEDIA_TYPE_AUDIO);
const audioCtx = AV.avformat_context(ic, audioStream);
AV.avcodec_open(audioCtx);

const videoStream = AV.av_find_best_stream(ic, AV.AVMEDIA_TYPE_VIDEO);
const videoCtx = videoStream === -1 ? null : AV.avcodec_open(ic, videoStream);
```
**Decode media file**
```js
let buf = audio_buffer.peek(1024 * 1024 * 4);
let ret = AV.avcodec_decode(ic, videoCtx, videoStream, audioCtx, audioStream, null, -1, buf);

```


## Caveat
- Only part of ffmpeg API is supported.
- Currently tested only on macos