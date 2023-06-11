#include "ScreenCap.h"
#include "ErrorException.h"
#include <string>
#include <iostream>

#define PrintWarning(str) \
	std::cout << "[WARNING] " << (str) << "!\n"

#define PrintInfo(str) \
	std::cout << "[INFO] " << (str) << "!\n"


FrameParams::FrameParams(int w, int h, AVPixelFormat fmt)
{
	width = w;
	height = h;
	pixFmt = fmt;
}

FrameParams& FrameParams::operator= (AVCodecParameters* params)
{
	if (params == nullptr)
		return *this;

	this->width = params->width;
	this->height = params->height;
	this->pixFmt = static_cast<AVPixelFormat>(params->format);
	return *this;
}




void Codec::openCodec(FrameParams IOFrameParams, AVCodecParameters* codecParams)
{
	m_frameCounter = 0;
}

void Codec::openFrameTransformer()
{
	if (m_swsCtx != nullptr)
		return;

	m_swsCtx = sws_getContext(
		m_inputFrameParams.width,
		m_inputFrameParams.height,
		m_inputFrameParams.pixFmt,
		m_outputFrameParams.width,
		m_outputFrameParams.height,
		m_outputFrameParams.pixFmt,
		SWS_FAST_BILINEAR,
		0, 0, 0
	);

	if (m_swsCtx == nullptr)
	{
		throw ERROR("Frame transformer was not initialized");
	}
}

void Codec::createFrameBuffer()
{
	if (m_frame != nullptr)
		return;

	m_frame = av_frame_alloc();

	if (m_frame == nullptr)
	{
		throw ERROR("Frame struct alloc error");
	}

	m_frame->format = m_outputFrameParams.pixFmt;
	m_frame->width = m_outputFrameParams.width;
	m_frame->height = m_outputFrameParams.height;


	int frameBufferSize = av_image_get_buffer_size(m_outputFrameParams.pixFmt, m_frame->width, m_frame->height, 1);
	m_frameBuffer = (uint8_t*)av_malloc(frameBufferSize);

	if (m_frameBuffer == nullptr)
	{
		throw ERROR("Frame buffer cannot be allocated");
	}

	int boundResult = av_image_fill_arrays(
		m_frame->data,
		m_frame->linesize,
		m_frameBuffer,
		m_outputFrameParams.pixFmt,
		m_frame->width,
		m_frame->height,
		1
	);

	if (boundResult < 0)
	{
		throw ERROR("Unable to bound frame struct and frame buffer");
	}
}

void Codec::closeCodec()
{
	if (m_codecContext != nullptr)
	{
		avcodec_close(m_codecContext);
		avcodec_free_context(&m_codecContext);
	}
	/*else
	{
		PrintWarning("Looks like codec already closed");
	}*/
}

void Codec::closeFrameTransformer()
{
	if (m_swsCtx != nullptr)
	{
		sws_freeContext(m_swsCtx);
		m_swsCtx = nullptr;
	}
	/*else
	{
		PrintWarning("Looks like frame transformer already closed");
	}*/
}

void Codec::freeFrame()
{
	if (m_frameBuffer == nullptr)
	{
		//PrintWarning("Frame is already freed");
		return;
	}

	av_free(m_frameBuffer);
	m_frameBuffer = nullptr;
	av_frame_free(&m_frame);
}

void Codec::transformFrame(const AVFrame* srcFrame)
{
	sws_scale(
		m_swsCtx,
		srcFrame->data,
		srcFrame->linesize,
		0,
		srcFrame->height,
		m_frame->data,
		m_frame->linesize
	);
}

bool Codec::open(FrameParams IOFrameParams, AVCodecParameters* codecParams)
{
	if (m_isOpen)
	{
		close();
	}

	try
	{
		openCodec(IOFrameParams, codecParams);
		openFrameTransformer();
		createFrameBuffer();
	}
	catch (Error& ex)
	{
		std::cout << ex.what() << '\n';
		close();
		return false;
	}

	m_isOpen = true;
	return true;
}

void Codec::close()
{
	closeCodec();
	closeFrameTransformer();
	freeFrame();

	m_isOpen = false;
}

void Codec::resetFrameCounter()
{
	m_frameCounter = 0;
}

const AVCodecContext* Codec::getCodecContext() const
{
	return m_codecContext;
}

Codec::~Codec()
{
	close();
}



void Encoder::openCodec(FrameParams inputFrameParams, AVCodecParameters* params)
{
	if (params == nullptr)
	{
		throw ERROR("Codec params are empty");
	}

	if (m_packet != nullptr)
	{
		return;
	}

	m_inputFrameParams = inputFrameParams;
	m_outputFrameParams = params;
	
	m_packet = av_packet_alloc();

	if (m_packet == nullptr)
	{
		throw ERROR("Packet struct alloc error");
	}

	m_codec = avcodec_find_encoder(params->codec_id);
	m_codecContext = avcodec_alloc_context3(m_codec);

	if (m_codecContext == nullptr)
	{
		throw ERROR("Codec context alloc error");
	}

	m_codecContext->bit_rate = 4000000;
	m_codecContext->width = m_outputFrameParams.width;
	m_codecContext->height = m_outputFrameParams.height;
	m_codecContext->time_base = m_frameRate;
	m_codecContext->gop_size = 10;
	m_codecContext->max_b_frames = 1;
	m_codecContext->pix_fmt = m_outputFrameParams.pixFmt;
	m_codecContext->sample_aspect_ratio.num = 1;
	m_codecContext->sample_aspect_ratio.den = 1;

	av_opt_set(m_codecContext->priv_data, "preset", "fast", 0);

	int openCodecResult = avcodec_open2(m_codecContext, m_codec, nullptr);
	if (openCodecResult < 0)
	{
		throw ERROR("Codec was not opened");
	}

	Codec::openCodec(m_inputFrameParams);
}

void Encoder::closeCodec()
{
	if (m_packet != nullptr)
	{
		av_packet_unref(m_packet);
		av_packet_free(&m_packet);
	}

	Codec::closeCodec();
}

AVPacket* Encoder::encodeFrame(const AVFrame* srcFrame)
{
	if (!m_isOpen)
		return nullptr;

	av_packet_unref(m_packet);

	transformFrame(srcFrame);
	m_frame->pts = m_frameCounter++;

	avcodec_send_frame(m_codecContext, m_frame);
	avcodec_receive_packet(m_codecContext, m_packet);

	return m_packet;
}




void Decoder::openCodec(FrameParams outputFrameParams, AVCodecParameters* params)
{
	if (params == nullptr)
	{
		throw ERROR("Codec params are empty");
	}

	if (m_codecContext != nullptr)
	{
		return;
	}

	m_codec = avcodec_find_decoder(params->codec_id);
	m_codecContext = avcodec_alloc_context3(m_codec);
	if (m_codecContext == nullptr)
	{
		throw ERROR("Codec context alloc error");
	}

	m_inputFrameParams = params;
	m_outputFrameParams = outputFrameParams;

	m_codecContext->time_base = m_frameRate;
	avcodec_parameters_to_context(m_codecContext, params);
	int openCodecResult = avcodec_open2(m_codecContext, m_codec, nullptr);
	if (openCodecResult < 0)
	{
		throw ERROR("Codec was not opened");
	}

	Codec::openCodec(m_inputFrameParams);
}

const AVFrame* Decoder::decodeFrame(const AVPacket* srcPacket)
{
	if (!m_isOpen)
		return nullptr;

	AVFrame* tmpFrame = av_frame_alloc();

	int isPacketSent = avcodec_send_packet(m_codecContext, srcPacket);
	if (isPacketSent < 0)
	{
		PrintWarning("Couldn't send packet to decoder");
	}

	int ret = avcodec_receive_frame(m_codecContext, tmpFrame);
	if (ret < 0)
	{
		PrintWarning("Couldn't receive frame from decoder");
	}

	transformFrame(tmpFrame);
	av_frame_unref(tmpFrame);
	av_frame_free(&tmpFrame);

	return m_frame;
}

void Decoder::flushCodecBuffers()
{
	if (m_codecContext != nullptr)
		avcodec_flush_buffers(m_codecContext);
}




void VideoIO::close()
{
}

VideoIO::~VideoIO()
{
	close();
}




bool VideoWriter::openFile(const char* filename, AVCodecID codecID, FrameParams inputFrameParams, FrameParams outputFrameParams)
{
	if (m_isOpen)
	{
		close();
	}

	try
	{
		const AVOutputFormat* outputFormat = av_guess_format(0, filename, 0);
		
		if (!outputFormat)
		{
			throw ERROR("Output format guess error");
		}

		if (avformat_alloc_output_context2(&m_formatCtx, outputFormat, nullptr, filename) < 0)
		{
			throw ERROR("Output format ctx was not allocated");
		}
		
		AVCodecParameters* params = avcodec_parameters_alloc();
		params->codec_id = codecID;
		params->width = outputFrameParams.width;
		params->height = outputFrameParams.height;
		params->format = outputFrameParams.pixFmt;

		if (!m_encoder.open(inputFrameParams, params))
		{
			avcodec_parameters_free(&params);
			throw ERROR("Codec open error");
		}
		avcodec_parameters_free(&params);

		const AVStream* videoStream = avformat_new_stream(m_formatCtx, nullptr);
		if (videoStream == nullptr)
		{
			throw ERROR("Unable to create new stream");
		}

		avcodec_parameters_from_context(videoStream->codecpar, m_encoder.getCodecContext());

		if (avio_open(&m_formatCtx->pb, filename, AVIO_FLAG_WRITE) < 0)
		{
			throw ERROR("Unable to open file");
		}

		if (avformat_write_header(m_formatCtx, nullptr))
		{
			avio_close(m_formatCtx->pb);
			avformat_free_context(m_formatCtx);
			throw ERROR("Unable to write header");
		}
	}
	catch (Error& ex)
	{
		std::cout << ex.what() << '\n';
		close();
		return false;
	}

	m_isOpen = true;
	PrintInfo(std::string(std::string(m_formatCtx->url) + " file opened").c_str());

	return true;
}

void VideoWriter::writeFrame(const AVFrame* frame)
{
	AVPacket* encodedPacket = m_encoder.encodeFrame(frame);

	if (m_isOpen == false || encodedPacket == nullptr)
	{
		return;
	}

	if (encodedPacket->pts != AV_NOPTS_VALUE)
	{
		encodedPacket->pts = av_rescale_q(
								encodedPacket->pts,
								m_encoder.getCodecContext()->time_base,
								m_formatCtx->streams[0]->time_base
								);
	}
	if (encodedPacket->dts != AV_NOPTS_VALUE)
	{
		encodedPacket->dts = av_rescale_q(
								encodedPacket->dts,
								m_encoder.getCodecContext()->time_base,
								m_formatCtx->streams[0]->time_base
								);
	}

	encodedPacket->duration = av_rescale_q(
								1,
								m_encoder.getCodecContext()->time_base,
								m_formatCtx->streams[0]->time_base
								);

	av_write_frame(m_formatCtx, encodedPacket);
}

void VideoWriter::close()
{
	av_write_trailer(m_formatCtx);
	avio_close(m_formatCtx->pb);

	if (m_formatCtx != nullptr)
	{
		std::string url = m_formatCtx->url;
		avformat_free_context(m_formatCtx);
		PrintInfo(std::string(url + " file closed").c_str());
	}
	/*else
	{
		PrintWarning("Nothing to close");
	}*/

	m_encoder.close();
	m_isOpen = false;
}




bool VideoReader::openInput(const char* url, FrameParams outputFrameParams, const AVInputFormat* inputFormat, AVDictionary* inputOptions)
{
	if (m_isOpen)
	{
		close();
	}

	try
	{
		if (avformat_open_input(&m_formatCtx, url, inputFormat, &inputOptions))
		{
			av_dict_free(&inputOptions);
			throw ERROR("Unable to open input");
		}
		av_dict_free(&inputOptions);

		if (avformat_find_stream_info(m_formatCtx, nullptr) < 0)
		{
			throw ERROR("Unable to find stream info");
		}


		m_videoStream = -1;
		for (uint32_t i = 0; i < m_formatCtx->nb_streams; i++)
		{
			if (m_formatCtx->streams[i]->codecpar->codec_type == AVMEDIA_TYPE_VIDEO)
			{
				m_videoStream = i;
				break;
			}
		}

		if (m_videoStream == -1)
		{
			throw ERROR("Unable to find videostream");
		}

		if (!m_decoder.open(outputFrameParams, m_formatCtx->streams[m_videoStream]->codecpar))
		{
			throw ERROR("Codec open error");
		}

		av_dump_format(m_formatCtx, m_videoStream, url, 0);

		m_packet = av_packet_alloc();
		if (m_packet == nullptr)
		{
			throw ERROR("Unable to alloc packet");
		}
	}
	catch (Error& ex)
	{
		std::cout << ex.what() << '\n';
		close();
		return false;
	}

	const AVStream* stream = m_formatCtx->streams[m_videoStream];
	if (stream->nb_frames > 0 && stream->duration > 0)
		m_packetDuration = stream->duration / stream->nb_frames;
	else
		m_packetDuration = 0;

	m_isOpen = true;
	PrintInfo(std::string(std::string(m_formatCtx->url) + " input stream opened").c_str());

	return true;
}

const AVFrame* VideoReader::readFrame()
{
	if (!m_isOpen)
		return nullptr;

	av_packet_unref(m_packet);
	av_read_frame(m_formatCtx, m_packet);

	if (m_packet->stream_index != m_videoStream)
		return nullptr;

	return m_decoder.decodeFrame(m_packet);
}

void VideoReader::fastForward(double seconds)
{
	if (!m_packet)
		return;
	
	int64_t pos = m_packet->pts;
	if (pos < 0)
	{
		pos = m_formatCtx->streams[m_videoStream]->duration;
	}

	int64_t timestamp = pos + seconds * m_formatCtx->streams[m_videoStream]->r_frame_rate.num * m_packetDuration;

	av_seek_frame(m_formatCtx, m_videoStream, timestamp, 0);
	m_decoder.flushCodecBuffers();
}

void VideoReader::close()
{
	if (m_packet != nullptr)
	{
		av_packet_unref(m_packet);
		av_packet_free(&m_packet);
	}

	if (m_formatCtx != nullptr)
	{
		std::string url = m_formatCtx->url;
		avformat_close_input(&m_formatCtx);
		avformat_free_context(m_formatCtx);
		PrintInfo(std::string(url + " input stream closed").c_str());
	}
	/*else
	{
		PrintWarning("Nothing to close");
	}*/

	m_decoder.close();

	m_videoStream = -1;
	m_packetDuration = 0;
	m_isOpen = false;
}




void ScreenRecorder::captureScreen()
{
	m_captureStopMutex.lock();

	while (!m_isCaptureStopped)
	{
		m_frameRGB = m_desktop.readFrame();
		onFrameUpdate();
		m_videoWriter.writeFrame(m_frameRGB);
	}

	m_captureStopMutex.unlock();
}

bool ScreenRecorder::initCapture(int windowWidth, int windowHeight, int offsetX, int offsetY, int outputFrameWidth, int outputFrameHeight)
{
	if (m_isOpen)
	{
		stopCapture();
	}

	try
	{
		avdevice_register_all();

		const AVInputFormat* inputFormat = av_find_input_format("gdigrab");
		AVDictionary* inputOptions = nullptr;
		std::string videoSize = std::to_string(windowWidth) + 'x' + std::to_string(windowHeight);

		av_dict_set(&inputOptions, "framerate", "60", 0);
		av_dict_set(&inputOptions, "offset_x", std::to_string(offsetX).c_str(), 0);
		av_dict_set(&inputOptions, "offset_y", std::to_string(offsetY).c_str(), 0);
		av_dict_set(&inputOptions, "video_size", videoSize.c_str(), 0);
		av_dict_set(&inputOptions, "probesize", "74M", 0);

		if (!m_desktop.openInput(
			"desktop",
			FrameParams(windowWidth, windowHeight, AV_PIX_FMT_RGB24),
			inputFormat,
			inputOptions
		))
		{
			throw ERROR("Unable to capture from desktop");
		}

		if (!m_videoWriter.openFile(
			"Encoded.mp4",
			AV_CODEC_ID_H264,
			FrameParams(windowWidth, windowHeight, AV_PIX_FMT_RGB24),
			FrameParams(outputFrameWidth, outputFrameHeight, AV_PIX_FMT_YUV420P)
		))
		{
			throw ERROR("Unable to init videowriter");
		}
	}
	catch (Error& ex)
	{
		std::cout << ex.what() << '\n';
		stopCapture();
		return false;
	}

	m_isOpen = true;
	return true;
}

bool ScreenRecorder::startCapture()
{
	if (!m_isOpen)
	{
		return false;
	}

	m_screenCapThread = new std::thread([this]() {
		this->captureScreen();
		});

	if (m_screenCapThread == nullptr)
	{
		stopCapture();
		return false;
	}

	m_isCaptureStopped = false;
	return true;
}

void ScreenRecorder::stopCapture()
{
	if (!m_isOpen)
		return;

	m_isCaptureStopped = true;
	m_captureStopMutex.lock();

	m_desktop.close();
	m_videoWriter.close();

	m_captureStopMutex.unlock();

	if (m_screenCapThread != nullptr)
	{
		m_screenCapThread->join();
		delete m_screenCapThread;
		m_screenCapThread = nullptr;
	}

	m_isOpen = false;
}

const AVFrame* const ScreenRecorder::getFrameRGB()
{
	return m_frameRGB;
}

void ScreenRecorder::onFrameUpdate()
{
}

ScreenRecorder::~ScreenRecorder()
{
	stopCapture();
}
