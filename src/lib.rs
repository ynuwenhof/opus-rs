// Copyright 2016 Tad Hardesty
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! High-level bindings for libopus.
//!
//! Only brief descriptions are included here. For detailed information, consult
//! the [libopus documentation](https://opus-codec.org/docs/opus_api-1.1.2/).
#![warn(missing_docs)]

extern crate audiopus_sys as ffi;

use std::convert::TryFrom;
use std::ffi::CStr;
use std::marker::PhantomData;
use std::os::raw::c_int;

// ============================================================================
// Constants

/// The possible applications for the codec.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
#[repr(i32)]
pub enum Application {
	/// Best for most VoIP/videoconference applications where listening quality
	/// and intelligibility matter most.
	Voip = ffi::OPUS_APPLICATION_VOIP,
	/// Best for broadcast/high-fidelity application where the decoded audio
	/// should be as close as possible to the input.
	Audio = ffi::OPUS_APPLICATION_AUDIO,
	/// Only use when lowest-achievable latency is what matters most.
	LowDelay = ffi::OPUS_APPLICATION_RESTRICTED_LOWDELAY,
}

/// The available channel setings.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Channels {
	/// One channel.
	Mono = 1,
	/// Two channels, left and right.
	Stereo = 2,
}

/// The available bandwidth level settings.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
#[repr(i32)]
pub enum Bandwidth {
	/// Auto/default setting.
	Auto = ffi::OPUS_AUTO,
	/// 4kHz bandpass.
	Narrowband = ffi::OPUS_BANDWIDTH_NARROWBAND,
	/// 6kHz bandpass.
	Mediumband = ffi::OPUS_BANDWIDTH_MEDIUMBAND,
	/// 8kHz bandpass.
	Wideband = ffi::OPUS_BANDWIDTH_WIDEBAND,
	/// 12kHz bandpass.
	Superwideband = ffi::OPUS_BANDWIDTH_SUPERWIDEBAND,
	/// 20kHz bandpass.
	Fullband = ffi::OPUS_BANDWIDTH_FULLBAND,
}

impl Bandwidth {
	fn from_int(value: i32) -> Option<Bandwidth> {
		Some(match value {
			ffi::OPUS_AUTO => Bandwidth::Auto,
			ffi::OPUS_BANDWIDTH_NARROWBAND => Bandwidth::Narrowband,
			ffi::OPUS_BANDWIDTH_MEDIUMBAND => Bandwidth::Mediumband,
			ffi::OPUS_BANDWIDTH_WIDEBAND => Bandwidth::Wideband,
			ffi::OPUS_BANDWIDTH_SUPERWIDEBAND => Bandwidth::Superwideband,
			ffi::OPUS_BANDWIDTH_FULLBAND => Bandwidth::Fullband,
			_ => return None,
		})
	}

	fn decode(value: i32, what: &'static str) -> Result<Bandwidth> {
		match Bandwidth::from_int(value) {
			Some(bandwidth) => Ok(bandwidth),
			None => Err(Error::bad_arg(what)),
		}
	}
}

/// Possible error codes.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[repr(i32)]
pub enum ErrorCode {
	/// One or more invalid/out of range arguments.
	BadArg = ffi::OPUS_BAD_ARG,
	/// Not enough bytes allocated in the buffer.
	BufferTooSmall = ffi::OPUS_BUFFER_TOO_SMALL,
	/// An internal error was detected.
	InternalError = ffi::OPUS_INTERNAL_ERROR,
	/// The compressed data passed is corrupted.
	InvalidPacket = ffi::OPUS_INVALID_PACKET,
	/// Invalid/unsupported request number.
	Unimplemented = ffi::OPUS_UNIMPLEMENTED,
	/// An encoder or decoder structure is invalid or already freed.
	InvalidState = ffi::OPUS_INVALID_STATE,
	/// Memory allocation has failed.
	AllocFail = ffi::OPUS_ALLOC_FAIL,
	/// An unknown failure.
	Unknown = -8,
}

impl ErrorCode {
	fn from_int(value: c_int) -> ErrorCode {
		use ErrorCode::*;
		match value {
			ffi::OPUS_BAD_ARG => BadArg,
			ffi::OPUS_BUFFER_TOO_SMALL => BufferTooSmall,
			ffi::OPUS_INTERNAL_ERROR => InternalError,
			ffi::OPUS_INVALID_PACKET => InvalidPacket,
			ffi::OPUS_UNIMPLEMENTED => Unimplemented,
			ffi::OPUS_INVALID_STATE => InvalidState,
			ffi::OPUS_ALLOC_FAIL => AllocFail,
			_ => Unknown,
		}
	}

	/// Get a human-readable error string for this error code.
	pub fn description(self) -> &'static str {
		// should always be ASCII and non-null for any input
		unsafe { CStr::from_ptr(ffi::opus_strerror(self as c_int)) }.to_str().unwrap()
	}
}

/// Possible bitrates.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Bitrate {
	/// Explicit bitrate choice (in bits/second).
	Bits(i32),
	/// Maximum bitrate allowed (up to maximum number of bytes for the packet).
	Max,
	/// Default bitrate decided by the encoder (not recommended).
	Auto,
}

impl Bitrate {
	fn from_raw(raw: c_int) -> Result<Bitrate> {
		Ok(match raw {
			ffi::OPUS_AUTO => Bitrate::Auto,
			ffi::OPUS_BITRATE_MAX => Bitrate::Max,
			_ => Bitrate::Bits(raw),
		})
	}

	fn raw(self) -> c_int {
		match self {
			Bitrate::Auto => ffi::OPUS_AUTO,
			Bitrate::Max => ffi::OPUS_BITRATE_MAX,
			Bitrate::Bits(raw) => raw,
		}
	}
}

/// Get the libopus version string.
///
/// Applications may look for the substring "-fixed" in the version string to
/// determine whether they have a fixed-point or floating-point build at
/// runtime.
pub fn version() -> &'static str {
	// verison string should always be ASCII
	unsafe { CStr::from_ptr(ffi::opus_get_version_string()) }.to_str().unwrap()
}

macro_rules! ffi {
	($f:ident $(, $rest:expr)*) => {
		match unsafe { ffi::$f($($rest),*) } {
			code if code < 0 => return Err(Error::from_code(stringify!($f), code)),
			code => code,
		}
	}
}

macro_rules! ctl {
	($f:ident, $this:ident, $ctl:path, $($rest:expr),*) => {
		match unsafe { ffi::$f($this.ptr, $ctl, $($rest),*) } {
			code if code < 0 => return Err(Error::from_code(
				concat!(stringify!($f), "(", stringify!($ctl), ")"),
				code,
			)),
			_ => (),
		}
	}
}

// ============================================================================
// Encoder

macro_rules! enc_ctl {
	($this:ident, $ctl:path $(, $rest:expr)*) => {
		ctl!(opus_encoder_ctl, $this, $ctl, $($rest),*)
	}
}

/// An Opus encoder with associated state.
#[derive(Debug)]
pub struct Encoder {
	ptr: *mut ffi::OpusEncoder,
	channels: Channels,
}

impl Encoder {
	/// Create and initialize an encoder.
	pub fn new(sample_rate: u32, channels: Channels, mode: Application) -> Result<Encoder> {
		let mut error = 0;
		let ptr = unsafe {
			ffi::opus_encoder_create(
				sample_rate as i32,
				channels as c_int,
				mode as c_int,
				&mut error,
			)
		};
		if error != ffi::OPUS_OK || ptr.is_null() {
			Err(Error::from_code("opus_encoder_create", error))
		} else {
			Ok(Encoder { ptr, channels })
		}
	}

	/// Encode an Opus frame.
	pub fn encode(&mut self, input: &[i16], output: &mut [u8]) -> Result<usize> {
		let len = ffi!(
			opus_encode,
			self.ptr,
			input.as_ptr(),
			len(input) / self.channels as c_int,
			output.as_mut_ptr(),
			len(output)
		);
		Ok(len as usize)
	}

	/// Encode an Opus frame from floating point input.
	pub fn encode_float(&mut self, input: &[f32], output: &mut [u8]) -> Result<usize> {
		let len = ffi!(
			opus_encode_float,
			self.ptr,
			input.as_ptr(),
			len(input) / self.channels as c_int,
			output.as_mut_ptr(),
			len(output)
		);
		Ok(len as usize)
	}

	/// Encode an Opus frame to a new buffer.
	pub fn encode_vec(&mut self, input: &[i16], max_size: usize) -> Result<Vec<u8>> {
		let mut output: Vec<u8> = vec![0; max_size];
		let result = self.encode(input, output.as_mut_slice())?;
		output.truncate(result);
		Ok(output)
	}

	/// Encode an Opus frame from floating point input to a new buffer.
	pub fn encode_vec_float(&mut self, input: &[f32], max_size: usize) -> Result<Vec<u8>> {
		let mut output: Vec<u8> = vec![0; max_size];
		let result = self.encode_float(input, output.as_mut_slice())?;
		output.truncate(result);
		Ok(output)
	}

	// ------------
	// Generic CTLs

	/// Reset the codec state to be equivalent to a freshly initialized state.
	pub fn reset_state(&mut self) -> Result<()> {
		enc_ctl!(self, ffi::OPUS_RESET_STATE);
		Ok(())
	}

	/// Get the final range of the codec's entropy coder.
	pub fn get_final_range(&mut self) -> Result<u32> {
		let mut value: u32 = 0;
		enc_ctl!(self, ffi::OPUS_GET_FINAL_RANGE_REQUEST, &mut value);
		Ok(value)
	}

	/// Get the encoder's configured bandpass.
	pub fn get_bandwidth(&mut self) -> Result<Bandwidth> {
		let mut value: i32 = 0;
		enc_ctl!(self, ffi::OPUS_GET_BANDWIDTH_REQUEST, &mut value);
		Bandwidth::decode(value, "opus_encoder_ctl(OPUS_GET_BANDWIDTH)")
	}

	/// Get the samping rate the encoder was intialized with.
	pub fn get_sample_rate(&mut self) -> Result<u32> {
		let mut value: i32 = 0;
		enc_ctl!(self, ffi::OPUS_GET_SAMPLE_RATE_REQUEST, &mut value);
		Ok(value as u32)
	}

	// ------------
	// Encoder CTLs

	/// Set the encoder's bitrate.
	pub fn set_bitrate(&mut self, value: Bitrate) -> Result<()> {
		enc_ctl!(self, ffi::OPUS_SET_BITRATE_REQUEST, value.raw());
		Ok(())
	}

	/// Get the encoder's bitrate.
	pub fn get_bitrate(&mut self) -> Result<Bitrate> {
		let mut value: i32 = 0;
		enc_ctl!(self, ffi::OPUS_GET_BITRATE_REQUEST, &mut value);
		Bitrate::from_raw(value)
	}

	/// Enable or disable variable bitrate.
	pub fn set_vbr(&mut self, vbr: bool) -> Result<()> {
		let value: i32 = if vbr { 1 } else { 0 };
		enc_ctl!(self, ffi::OPUS_SET_VBR_REQUEST, value);
		Ok(())
	}

	/// Determine if variable bitrate is enabled.
	pub fn get_vbr(&mut self) -> Result<bool> {
		let mut value: i32 = 0;
		enc_ctl!(self, ffi::OPUS_GET_VBR_REQUEST, &mut value);
		Ok(value != 0)
	}

	/// Enable or disable constrained VBR.
	pub fn set_vbr_constraint(&mut self, vbr: bool) -> Result<()> {
		let value: i32 = if vbr { 1 } else { 0 };
		enc_ctl!(self, ffi::OPUS_SET_VBR_CONSTRAINT_REQUEST, value);
		Ok(())
	}

	/// Determine if constrained VBR is enabled.
	pub fn get_vbr_constraint(&mut self) -> Result<bool> {
		let mut value: i32 = 0;
		enc_ctl!(self, ffi::OPUS_GET_VBR_CONSTRAINT_REQUEST, &mut value);
		Ok(value != 0)
	}

	/// Configures the encoder's use of inband forward error correction (FEC).
	pub fn set_inband_fec(&mut self, value: bool) -> Result<()> {
		let value: i32 = if value { 1 } else { 0 };
		enc_ctl!(self, ffi::OPUS_SET_INBAND_FEC_REQUEST, value);
		Ok(())
	}

	/// Gets encoder's configured use of inband forward error correction.
	pub fn get_inband_fec(&mut self) -> Result<bool> {
		let mut value: i32 = 0;
		enc_ctl!(self, ffi::OPUS_GET_INBAND_FEC_REQUEST, &mut value);
		Ok(value != 0)
	}

	/// Configures the encoder's use of discontinuous transmission (DTX).
	pub fn set_dtx(&mut self, value: bool) -> Result<()> {
		let value: i32 = if value { 1 } else { 0 };
		enc_ctl!(self, ffi::OPUS_SET_DTX_REQUEST, value);
		Ok(())
	}

	/// Gets encoder's configured use of discontinuous transmission (DTX).
	pub fn get_dtx(&mut self) -> Result<bool> {
		let mut value: i32 = 0;
		enc_ctl!(self, ffi::OPUS_GET_DTX_REQUEST, &mut value);
		Ok(value != 0)
	}

	/// Configures the encoder's computational complexity.
	pub fn set_complexity(&mut self, value: i32) -> Result<()> {
		enc_ctl!(self, ffi::OPUS_SET_COMPLEXITY_REQUEST, value);
		Ok(())
	}

	/// Gets the encoder's complexity configuration.
	pub fn get_complexity(&mut self) -> Result<i32> {
		let mut value: i32 = 0;
		enc_ctl!(self, ffi::OPUS_GET_COMPLEXITY_REQUEST, &mut value);
		Ok(value)
	}

	/// Sets the encoder's expected packet loss percentage.
	pub fn set_packet_loss_perc(&mut self, value: i32) -> Result<()> {
		enc_ctl!(self, ffi::OPUS_SET_PACKET_LOSS_PERC_REQUEST, value);
		Ok(())
	}

	/// Gets the encoder's expected packet loss percentage.
	pub fn get_packet_loss_perc(&mut self) -> Result<i32> {
		let mut value: i32 = 0;
		enc_ctl!(self, ffi::OPUS_GET_PACKET_LOSS_PERC_REQUEST, &mut value);
		Ok(value)
	}

	/// Gets the total samples of delay added by the entire codec.
	pub fn get_lookahead(&mut self) -> Result<i32> {
		let mut value: i32 = 0;
		enc_ctl!(self, ffi::OPUS_GET_LOOKAHEAD_REQUEST, &mut value);
		Ok(value)
	}

	/// Configures mono/stereo forcing in the encoder.
	///
	/// This can force the encoder to produce packets encoded as either mono or
	/// stereo, regardless of the format of the input audio. This is useful
	/// when the caller knows that the input signal is currently a mono source
	/// embedded in a stereo stream.
	pub fn set_force_channels(&mut self, value: Option<Channels>) -> Result<()> {
		let value = match value {
			None => ffi::OPUS_AUTO,
			Some(Channels::Mono) => 1,
			Some(Channels::Stereo) => 2,
		};
		enc_ctl!(self, ffi::OPUS_SET_FORCE_CHANNELS_REQUEST, value);
		Ok(())
	}

	/// Gets the encoder's forced channel configuration.
	pub fn get_force_channels(&mut self) -> Result<Option<Channels>> {
		let mut value: i32 = 0;
		enc_ctl!(self, ffi::OPUS_GET_FORCE_CHANNELS_REQUEST, &mut value);
		match value {
			ffi::OPUS_AUTO => Ok(None),
			1 => Ok(Some(Channels::Mono)),
			2 => Ok(Some(Channels::Stereo)),
			_ => Err(Error::bad_arg("opus_encoder_ctl(OPUS_GET_FORCE_CHANNELS)")),
		}
	}

	// TODO: Encoder-specific CTLs
}

impl Drop for Encoder {
	fn drop(&mut self) {
		unsafe { ffi::opus_encoder_destroy(self.ptr) }
	}
}

// "A single codec state may only be accessed from a single thread at
// a time and any required locking must be performed by the caller. Separate
// streams must be decoded with separate decoder states and can be decoded
// in parallel unless the library was compiled with NONTHREADSAFE_PSEUDOSTACK
// defined."
//
// In other words, opus states may be moved between threads at will. A special
// compilation mode intended for embedded platforms forbids multithreaded use
// of the library as a whole rather than on a per-state basis, but the opus-sys
// crate does not use this mode.
unsafe impl Send for Encoder {}

// ============================================================================
// Decoder

macro_rules! dec_ctl {
	($this:ident, $ctl:path $(, $rest:expr)*) => {
		ctl!(opus_decoder_ctl, $this, $ctl, $($rest),*)
	}
}

/// An Opus decoder with associated state.
#[derive(Debug)]
pub struct Decoder {
	ptr: *mut ffi::OpusDecoder,
	channels: Channels,
}

impl Decoder {
	/// Create and initialize a decoder.
	pub fn new(sample_rate: u32, channels: Channels) -> Result<Decoder> {
		let mut error = 0;
		let ptr =
			unsafe { ffi::opus_decoder_create(sample_rate as i32, channels as c_int, &mut error) };
		if error != ffi::OPUS_OK || ptr.is_null() {
			Err(Error::from_code("opus_decoder_create", error))
		} else {
			Ok(Decoder { ptr, channels })
		}
	}

	/// Decode an Opus packet.
	///
	/// To represent packet loss, pass an empty slice `&[]`.
	///
	/// The return value is the number of samples *per channel* decoded from
	/// the packet.
	pub fn decode(&mut self, input: &[u8], output: &mut [i16], fec: bool) -> Result<usize> {
		let ptr = match input.len() {
			0 => std::ptr::null(),
			_ => input.as_ptr(),
		};
		let len = ffi!(
			opus_decode,
			self.ptr,
			ptr,
			len(input),
			output.as_mut_ptr(),
			len(output) / self.channels as c_int,
			fec as c_int
		);
		Ok(len as usize)
	}

	/// Decode an Opus packet with floating point output.
	///
	/// To represent packet loss, pass an empty slice `&[]`.
	///
	/// The return value is the number of samples *per channel* decoded from
	/// the packet.
	pub fn decode_float(&mut self, input: &[u8], output: &mut [f32], fec: bool) -> Result<usize> {
		let ptr = match input.len() {
			0 => std::ptr::null(),
			_ => input.as_ptr(),
		};
		let len = ffi!(
			opus_decode_float,
			self.ptr,
			ptr,
			len(input),
			output.as_mut_ptr(),
			len(output) / self.channels as c_int,
			fec as c_int
		);
		Ok(len as usize)
	}

	/// Get the number of samples *per channel* of an Opus packet.
	pub fn get_nb_samples(&self, packet: &[u8]) -> Result<usize> {
		let len = ffi!(opus_decoder_get_nb_samples, self.ptr, packet.as_ptr(), packet.len() as i32);
		Ok(len as usize)
	}

	// ------------
	// Generic CTLs

	/// Reset the codec state to be equivalent to a freshly initialized state.
	pub fn reset_state(&mut self) -> Result<()> {
		dec_ctl!(self, ffi::OPUS_RESET_STATE);
		Ok(())
	}

	/// Get the final range of the codec's entropy coder.
	pub fn get_final_range(&mut self) -> Result<u32> {
		let mut value: u32 = 0;
		dec_ctl!(self, ffi::OPUS_GET_FINAL_RANGE_REQUEST, &mut value);
		Ok(value)
	}

	/// Get the decoder's last bandpass.
	pub fn get_bandwidth(&mut self) -> Result<Bandwidth> {
		let mut value: i32 = 0;
		dec_ctl!(self, ffi::OPUS_GET_BANDWIDTH_REQUEST, &mut value);
		Bandwidth::decode(value, "opus_decoder_ctl(OPUS_GET_BANDWIDTH)")
	}

	/// Get the samping rate the decoder was intialized with.
	pub fn get_sample_rate(&mut self) -> Result<u32> {
		let mut value: i32 = 0;
		dec_ctl!(self, ffi::OPUS_GET_SAMPLE_RATE_REQUEST, &mut value);
		Ok(value as u32)
	}

	// ------------
	// Decoder CTLs

	/// Configures decoder gain adjustment.
	///
	/// Scales the decoded output by a factor specified in Q8 dB units. This has
	/// a maximum range of -32768 to 32768 inclusive, and returns `BadArg`
	/// otherwise. The default is zero indicating no adjustment. This setting
	/// survives decoder reset.
	///
	/// `gain = pow(10, x / (20.0 * 256))`
	pub fn set_gain(&mut self, gain: i32) -> Result<()> {
		dec_ctl!(self, ffi::OPUS_SET_GAIN_REQUEST, gain);
		Ok(())
	}

	/// Gets the decoder's configured gain adjustment.
	pub fn get_gain(&mut self) -> Result<i32> {
		let mut value: i32 = 0;
		dec_ctl!(self, ffi::OPUS_GET_GAIN_REQUEST, &mut value);
		Ok(value)
	}

	/// Gets the duration (in samples) of the last packet successfully decoded
	/// or concealed.
	pub fn get_last_packet_duration(&mut self) -> Result<u32> {
		let mut value: i32 = 0;
		dec_ctl!(self, ffi::OPUS_GET_LAST_PACKET_DURATION_REQUEST, &mut value);
		Ok(value as u32)
	}

	/// Gets the pitch of the last decoded frame, if available.
	///
	/// This can be used for any post-processing algorithm requiring the use of
	/// pitch, e.g. time stretching/shortening. If the last frame was not
	/// voiced, or if the pitch was not coded in the frame, then zero is
	/// returned.
	pub fn get_pitch(&mut self) -> Result<i32> {
		let mut value: i32 = 0;
		dec_ctl!(self, ffi::OPUS_GET_PITCH_REQUEST, &mut value);
		Ok(value)
	}
}

impl Drop for Decoder {
	fn drop(&mut self) {
		unsafe { ffi::opus_decoder_destroy(self.ptr) }
	}
}

// See `unsafe impl Send for Encoder`.
unsafe impl Send for Decoder {}

// ============================================================================
// Packet Analysis

/// Analyze raw Opus packets.
pub mod packet {
	use super::ffi;
	use super::*;
	use std::{ptr, slice};

	/// Get the bandwidth of an Opus packet.
	pub fn get_bandwidth(packet: &[u8]) -> Result<Bandwidth> {
		if packet.is_empty() {
			return Err(Error::bad_arg("opus_packet_get_bandwidth"));
		}
		let bandwidth = ffi!(opus_packet_get_bandwidth, packet.as_ptr());
		Bandwidth::decode(bandwidth, "opus_packet_get_bandwidth")
	}

	/// Get the number of channels from an Opus packet.
	pub fn get_nb_channels(packet: &[u8]) -> Result<Channels> {
		if packet.is_empty() {
			return Err(Error::bad_arg("opus_packet_get_nb_channels"));
		}
		let channels = ffi!(opus_packet_get_nb_channels, packet.as_ptr());
		match channels {
			1 => Ok(Channels::Mono),
			2 => Ok(Channels::Stereo),
			_ => Err(Error::bad_arg("opus_packet_get_nb_channels")),
		}
	}

	/// Get the number of frames in an Opus packet.
	pub fn get_nb_frames(packet: &[u8]) -> Result<usize> {
		let frames = ffi!(opus_packet_get_nb_frames, packet.as_ptr(), len(packet));
		Ok(frames as usize)
	}

	/// Get the number of samples of an Opus packet.
	pub fn get_nb_samples(packet: &[u8], sample_rate: u32) -> Result<usize> {
		let frames =
			ffi!(opus_packet_get_nb_samples, packet.as_ptr(), len(packet), sample_rate as c_int);
		Ok(frames as usize)
	}

	/// Get the number of samples per frame from an Opus packet.
	pub fn get_samples_per_frame(packet: &[u8], sample_rate: u32) -> Result<usize> {
		if packet.is_empty() {
			return Err(Error::bad_arg("opus_packet_get_samples_per_frame"));
		}
		let samples =
			ffi!(opus_packet_get_samples_per_frame, packet.as_ptr(), sample_rate as c_int);
		Ok(samples as usize)
	}

	/// Parse an Opus packet into one or more frames.
	pub fn parse(packet: &[u8]) -> Result<Packet> {
		let mut toc: u8 = 0;
		let mut frames = [ptr::null(); 48];
		let mut sizes = [0i16; 48];
		let mut payload_offset: i32 = 0;
		let num_frames = ffi!(
			opus_packet_parse,
			packet.as_ptr(),
			len(packet),
			&mut toc,
			frames.as_mut_ptr(),
			sizes.as_mut_ptr(),
			&mut payload_offset
		);

		let mut frames_vec = Vec::with_capacity(num_frames as usize);
		for i in 0..num_frames as usize {
			frames_vec.push(unsafe { slice::from_raw_parts(frames[i], sizes[i] as usize) });
		}

		Ok(Packet {
			toc,
			frames: frames_vec,
			payload_offset: payload_offset as usize,
		})
	}

	/// A parsed Opus packet, retuned from `parse`.
	#[derive(Debug)]
	pub struct Packet<'a> {
		/// The TOC byte of the packet.
		pub toc: u8,
		/// The frames contained in the packet.
		pub frames: Vec<&'a [u8]>,
		/// The offset into the packet at which the payload is located.
		pub payload_offset: usize,
	}

	/// Pad a given Opus packet to a larger size.
	///
	/// The packet will be extended from the first `prev_len` bytes of the
	/// buffer into the rest of the available space.
	pub fn pad(packet: &mut [u8], prev_len: usize) -> Result<usize> {
		let result = ffi!(opus_packet_pad, packet.as_mut_ptr(), check_len(prev_len), len(packet));
		Ok(result as usize)
	}

	/// Remove all padding from a given Opus packet and rewrite the TOC sequence
	/// to minimize space usage.
	pub fn unpad(packet: &mut [u8]) -> Result<usize> {
		let result = ffi!(opus_packet_unpad, packet.as_mut_ptr(), len(packet));
		Ok(result as usize)
	}
}

// ============================================================================
// Float Soft Clipping

/// Soft-clipping to bring a float signal within the [-1,1] range.
#[derive(Debug)]
pub struct SoftClip {
	channels: Channels,
	memory: [f32; 2],
}

impl SoftClip {
	/// Initialize a new soft-clipping state.
	pub fn new(channels: Channels) -> SoftClip {
		SoftClip { channels, memory: [0.0; 2] }
	}

	/// Apply soft-clipping to a float signal.
	pub fn apply(&mut self, signal: &mut [f32]) {
		unsafe {
			ffi::opus_pcm_soft_clip(
				signal.as_mut_ptr(),
				len(signal) / self.channels as c_int,
				self.channels as c_int,
				self.memory.as_mut_ptr(),
			)
		};
	}
}

// ============================================================================
// Repacketizer

/// A repacketizer used to merge together or split apart multiple Opus packets.
#[derive(Debug)]
pub struct Repacketizer {
	ptr: *mut ffi::OpusRepacketizer,
}

impl Repacketizer {
	/// Create and initialize a repacketizer.
	pub fn new() -> Result<Repacketizer> {
		let ptr = unsafe { ffi::opus_repacketizer_create() };
		if ptr.is_null() {
			Err(Error::from_code("opus_repacketizer_create", ffi::OPUS_ALLOC_FAIL))
		} else {
			Ok(Repacketizer { ptr })
		}
	}

	/// Shortcut to combine several smaller packets into one larger one.
	pub fn combine(&mut self, input: &[&[u8]], output: &mut [u8]) -> Result<usize> {
		let mut state = self.begin();
		for &packet in input {
			state.cat(packet)?;
		}
		state.out(output)
	}

	/// Begin using the repacketizer.
	#[allow(clippy::needless_lifetimes)]
	pub fn begin<'rp, 'buf>(&'rp mut self) -> RepacketizerState<'rp, 'buf> {
		unsafe {
			ffi::opus_repacketizer_init(self.ptr);
		}
		RepacketizerState { rp: self, phantom: PhantomData }
	}
}

impl Drop for Repacketizer {
	fn drop(&mut self) {
		unsafe { ffi::opus_repacketizer_destroy(self.ptr) }
	}
}

// See `unsafe impl Send for Encoder`.
unsafe impl Send for Repacketizer {}

// To understand why these lifetime bounds are needed, imagine that the
// repacketizer keeps an internal Vec<&'buf [u8]>, which is added to by cat()
// and accessed by get_nb_frames(), out(), and out_range(). To prove that these
// lifetime bounds are correct, a dummy implementation with the same signatures
// but a real Vec<&'buf [u8]> rather than unsafe blocks may be substituted.

/// An in-progress repacketization.
#[derive(Debug)]
pub struct RepacketizerState<'rp, 'buf> {
	rp: &'rp mut Repacketizer,
	phantom: PhantomData<&'buf [u8]>,
}

impl<'rp, 'buf> RepacketizerState<'rp, 'buf> {
	/// Add a packet to the current repacketizer state.
	pub fn cat(&mut self, packet: &'buf [u8]) -> Result<()> {
		ffi!(opus_repacketizer_cat, self.rp.ptr, packet.as_ptr(), len(packet));
		Ok(())
	}

	/// Add a packet to the current repacketizer state, moving it.
	#[inline]
	pub fn cat_move<'b2>(self, packet: &'b2 [u8]) -> Result<RepacketizerState<'rp, 'b2>>
	where
		'buf: 'b2,
	{
		let mut shorter = self;
		shorter.cat(packet)?;
		Ok(shorter)
	}

	/// Get the total number of frames contained in packet data submitted so
	/// far via `cat`.
	pub fn get_nb_frames(&mut self) -> usize {
		unsafe { ffi::opus_repacketizer_get_nb_frames(self.rp.ptr) as usize }
	}

	/// Construct a new packet from data previously submitted via `cat`.
	///
	/// All previously submitted frames are used.
	pub fn out(&mut self, buffer: &mut [u8]) -> Result<usize> {
		let result = ffi!(opus_repacketizer_out, self.rp.ptr, buffer.as_mut_ptr(), len(buffer));
		Ok(result as usize)
	}

	/// Construct a new packet from data previously submitted via `cat`, with
	/// a manually specified subrange.
	///
	/// The `end` index should not exceed the value of `get_nb_frames()`.
	pub fn out_range(&mut self, begin: usize, end: usize, buffer: &mut [u8]) -> Result<usize> {
		let result = ffi!(
			opus_repacketizer_out_range,
			self.rp.ptr,
			check_len(begin),
			check_len(end),
			buffer.as_mut_ptr(),
			len(buffer)
		);
		Ok(result as usize)
	}
}

// ============================================================================
// TODO: Multistream API

// ============================================================================
// Error Handling

/// Opus error Result alias.
pub type Result<T> = std::result::Result<T, Error>;

/// An error generated by the Opus library.
#[derive(Debug)]
pub struct Error {
	function: &'static str,
	code: ErrorCode,
}

impl Error {
	fn bad_arg(what: &'static str) -> Error {
		Error { function: what, code: ErrorCode::BadArg }
	}

	fn from_code(what: &'static str, code: c_int) -> Error {
		Error {
			function: what,
			code: ErrorCode::from_int(code),
		}
	}

	/// Get the name of the Opus function from which the error originated.
	#[inline]
	pub fn function(&self) -> &'static str {
		self.function
	}

	/// Get a textual description of the error provided by Opus.
	#[inline]
	pub fn description(&self) -> &'static str {
		self.code.description()
	}

	/// Get the Opus error code of the error.
	#[inline]
	pub fn code(&self) -> ErrorCode {
		self.code
	}
}

impl std::fmt::Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "{}: {}", self.function, self.description())
	}
}

impl std::error::Error for Error {
	fn description(&self) -> &str {
		self.code.description()
	}
}

fn check_len(val: usize) -> c_int {
	match c_int::try_from(val) {
		Ok(val2) => val2,
		Err(_) => panic!("length out of range: {}", val),
	}
}

#[inline]
fn len<T>(slice: &[T]) -> c_int {
	check_len(slice.len())
}
