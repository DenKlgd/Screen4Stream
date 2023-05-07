#pragma once

extern "C"
{
	#include <libavcodec/avcodec.h>
	#include <libavformat/avformat.h>
	#include <libavdevice/avdevice.h>
	#include <libavutil/imgutils.h>
	#include <libswscale/swscale.h>
}

#include <thread>
#include <mutex>

//struct FrameUpdatedCallback
//{
//	void* callerPtr = nullptr;
//	void (*onFrameUpdate)() = nullptr;
//};

class ScreenRecorder
{
protected:
	AVFormatContext* formatCtx = nullptr;
	uint32_t videoStream = -1;
	AVCodecParameters* inputCodecParams = nullptr;
	AVCodecContext* inputCodecContext = nullptr;
	uint8_t* frameBuffer = nullptr;
	AVFrame* frameRGB = nullptr;
	AVFrame* frameBGRA = nullptr;
	SwsContext* swsCtx = nullptr;
	AVPacket* packet = nullptr;
	std::mutex captureStopMutex;
	bool isCaptureStopped = false;
	bool isInitialized = false;
	std::thread* screenCapThread = nullptr;

protected:
	int captureScreen();
	virtual void onFrameUpdate();

public:
	int initCapture(int windowWidth, int windowHeight, int offsetX, int offsetY, int outputFrameWidth, int outputFrameHeight);
	int startCapture();
	int stopCapture();
	const AVFrame* const getFrameRGB();
	const AVFrame* const getFrameBGRA();
	~ScreenRecorder();
};

