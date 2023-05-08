#include "ScreenCap.h"
#include <string>
#include <iostream>
#include <chrono>

int ScreenRecorder::captureScreen()
{
	if (!isInitialized)
		return -1;

	captureStopMutex.lock();
	isCaptureStopped = false;
	int frameFinished = true;
	int frameCount = 0;
	//std::chrono::steady_clock::time_point t0 = std::chrono::high_resolution_clock::now();

	while (av_read_frame(formatCtx, packet) >= 0)
	{
		if (packet->stream_index == videoStream)
		{
			int isPacketSent = avcodec_send_packet(inputCodecContext, packet);
			if (isPacketSent < 0)
			{
				std::cout << "Couldn't send packet to encoder - ERROR!\n";
				captureStopMutex.unlock();
				return -1;
			}

			int isFrameReceived = avcodec_receive_frame(inputCodecContext, frameBGRA);
			if (isFrameReceived < 0)
			{
				std::cout << "Couldn't receive frame from decoder - ERROR!\n";
				frameFinished = false;
			}

			if (frameFinished)
			{
				int scaleStatus = sws_scale(swsCtx,
					frameBGRA->data,
					frameBGRA->linesize,
					0,
					frameBGRA->height,
					frameRGB->data,
					frameRGB->linesize
				);

				if (scaleStatus < 0)
				{
					std::cout << "Couldn't sws_scale - ERROR!\n";
				}
				else
				{
					onFrameUpdate();
				}
				/*else
				{
					frameCount++;

					if (std::chrono::high_resolution_clock::now() - t0 >= std::chrono::seconds(1))
					{
						std::cout << "FPS: " << frameCount << '\n';
						frameCount = 0;
						t0 = std::chrono::high_resolution_clock::now();
					}
				}*/
			}
		}
		av_packet_unref(packet);

		if (isCaptureStopped)
		{
			break;
		}
	}

	isCaptureStopped = true;
	captureStopMutex.unlock();
	return 0;
}

int ScreenRecorder::initCapture(int windowWidth, int windowHeight, int offsetX, int offsetY, int outputFrameWidth, int outputFrameHeight)
{
	if (isInitialized)
		return -1;

	isCaptureStopped = true;

	avdevice_register_all();
	formatCtx = avformat_alloc_context();

	const AVInputFormat* inputFormat = av_find_input_format("gdigrab");
	AVDictionary* inputOptions = nullptr;
	std::string videoSize = std::to_string(windowWidth) + 'x' + std::to_string(windowHeight);

	av_dict_set(&inputOptions, "framerate", "60", NULL);
	av_dict_set(&inputOptions, "offset_x", std::to_string(offsetX).c_str(), NULL);
	av_dict_set(&inputOptions, "offset_y", std::to_string(offsetY).c_str(), NULL);
	av_dict_set(&inputOptions, "video_size", videoSize.c_str(), NULL);
	av_dict_set(&inputOptions, "probesize", "74M", NULL);


	if (avformat_open_input(&formatCtx, "desktop", inputFormat, &inputOptions))
	{
		std::cout << "avformat_open_input - ERROR!\n";
		return -1;
	}

	if (avformat_find_stream_info(formatCtx, NULL) < 0)
	{
		std::cout << "avformat_find_stream_info - ERROR!\n";
		return -1;
	}

	av_dump_format(formatCtx, 0, "desktop", 0);


	for (uint32_t i = 0; i < formatCtx->nb_streams; i++)
	{
		if (formatCtx->streams[i]->codecpar->codec_type == AVMEDIA_TYPE_VIDEO)
		{
			videoStream = i;
			break;
		}
	}

	if (videoStream == -1)
	{
		std::cout << "Couldn't find videostream - ERROR!\n";
		return -1;
	}


	inputCodecParams = formatCtx->streams[videoStream]->codecpar;
	const AVCodec* inputCodec = avcodec_find_decoder(inputCodecParams->codec_id);
	if (!inputCodec)
	{
		std::cout << "Couldn't find input codec - ERROR!\n";
		return -1;
	}


	inputCodecContext = avcodec_alloc_context3(inputCodec);
	if (!inputCodecContext)
	{
		std::cout << "Couldn't alloc input codec context - ERROR!\n";
		return -1;
	}

	if (avcodec_parameters_to_context(inputCodecContext, inputCodecParams))
	{
		std::cout << "Couldn't copy input codec params to context - ERROR!\n";
		return -1;
	}

	if (avcodec_open2(inputCodecContext, inputCodec, NULL) < 0)
	{
		std::cout << "Unable to open input codec - ERROR!\n";
		return -1;
	}


	frameRGB = av_frame_alloc();
	frameBGRA = av_frame_alloc();
	if (!frameRGB || !frameBGRA)
	{
		std::cout << "Couldn't alloc frame - ERROR!\n";
		return -1;
	}

	int picSize = av_image_get_buffer_size(
		AV_PIX_FMT_RGB24,
		outputFrameWidth, //inputCodecContext->width,
		outputFrameHeight, //inputCodecContext->height,
		1
	);

	frameBuffer = (uint8_t*)av_malloc(picSize);
	if (!frameBuffer)
	{
		std::cout << "Couldn't alloc buffer - ERROR!\n";
		return -1;
	}

	av_image_fill_arrays(
		frameRGB->data,
		frameRGB->linesize,
		frameBuffer,
		AV_PIX_FMT_RGB24,
		outputFrameWidth, //inputCodecContext->width,
		outputFrameHeight, //inputCodecContext->height,
		1
	);

	frameRGB->width = outputFrameWidth;
	frameRGB->height = outputFrameHeight;

	swsCtx = sws_getContext(
		inputCodecContext->width,
		inputCodecContext->height,
		inputCodecContext->pix_fmt,
		outputFrameWidth,//inputCodecContext->width,
		outputFrameHeight,//inputCodecContext->height,
		AV_PIX_FMT_RGB24,
		SWS_FAST_BILINEAR,
		nullptr,
		nullptr,
		nullptr
	);

	packet = av_packet_alloc();

	isInitialized = true;
	return 0;
}

int ScreenRecorder::startCapture()
{
	if (!isInitialized || screenCapThread != nullptr)
		return -1;

	screenCapThread = new std::thread([this]() {
		this->captureScreen();
		});

	if (screenCapThread == nullptr)
		return -1;

	return 0;
}

int ScreenRecorder::stopCapture()
{
	if (!isInitialized)
		return -1;

	isCaptureStopped = true;
	captureStopMutex.lock();

	sws_freeContext(swsCtx);
	av_packet_free(&packet);
	av_free(frameBuffer);
	av_frame_free(&frameBGRA);
	av_frame_free(&frameRGB);
	avcodec_free_context(&inputCodecContext);
	avformat_close_input(&formatCtx);
	avformat_free_context(formatCtx);

	isInitialized = false;
	captureStopMutex.unlock();

	if (screenCapThread != nullptr)
	{
		screenCapThread->join();
		delete screenCapThread;
		screenCapThread = nullptr;
	}

	return 0;
}

const AVFrame* const ScreenRecorder::getFrameRGB()
{
	return frameRGB;
}

const AVFrame* const ScreenRecorder::getFrameBGRA()
{
	return frameBGRA;
}

bool ScreenRecorder::isInit()
{
    return isInitialized;
}

bool ScreenRecorder::isCapturing()
{
	return !isCaptureStopped;
}

void ScreenRecorder::onFrameUpdate()
{
}

ScreenRecorder::~ScreenRecorder()
{
	stopCapture();
}
