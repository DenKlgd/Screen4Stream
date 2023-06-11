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


struct FrameParams
{
	int width = 0;
	int height = 0;
	AVPixelFormat pixFmt = AVPixelFormat::AV_PIX_FMT_NONE;

	FrameParams() = default;
	FrameParams(int w, int h, AVPixelFormat fmt);
	FrameParams& operator= (AVCodecParameters* params);
};



class Codec
{
protected:
	const AVCodec* m_codec = nullptr;
	AVCodecContext* m_codecContext = nullptr;
	AVCodecParameters* m_codecParams = nullptr;
	SwsContext* m_swsCtx = nullptr;
	uint8_t* m_frameBuffer = nullptr;
	AVFrame* m_frame = nullptr;
	AVRational m_frameRate = { 1, 30 };
	FrameParams m_inputFrameParams;
	FrameParams m_outputFrameParams;
	int64_t m_frameCounter = 0;
	bool m_isOpen = false;

private:
	void openFrameTransformer();
	void createFrameBuffer();
	void closeFrameTransformer();
	void freeFrame();

protected:
	virtual void openCodec(FrameParams IOFrameParams, AVCodecParameters* codecParams = nullptr);
	virtual void closeCodec();
	void transformFrame(const AVFrame* srcFrame);
	void resetFrameCounter();
	Codec() = default;

public:
	bool open(FrameParams IOFrameParams, AVCodecParameters* codecParams = nullptr);
	void close();
	const AVCodecContext* getCodecContext() const;
	virtual ~Codec();
};


class Encoder : public Codec
{
protected:
	AVPacket* m_packet = nullptr;

protected:
	void openCodec(FrameParams inputFrameParams, AVCodecParameters* codecParams = nullptr) override;
	void closeCodec() override;

public:
	AVPacket* encodeFrame(const AVFrame* srcFrame);
	virtual ~Encoder() = default;
};


class Decoder : public Codec
{
protected:
	void openCodec(FrameParams outputFrameParams, AVCodecParameters* codecParams = nullptr) override;

public:
	const AVFrame* decodeFrame(const AVPacket* srcPacket);
	void flushCodecBuffers();
	virtual ~Decoder() = default;
};


class VideoIO
{
protected:
	AVFormatContext* m_formatCtx = nullptr;
	bool m_isOpen = false;

public:
	virtual void close();
	virtual ~VideoIO();
};


class VideoWriter : public VideoIO
{
private:
	Encoder m_encoder;

public:
	bool openFile(const char* filename, AVCodecID codecID, FrameParams inputFrameParams, FrameParams outputFrameParams);
	void writeFrame(const AVFrame* frame);
	void close() override;
};


class VideoReader : public VideoIO
{
private:
	Decoder m_decoder;
	AVPacket* m_packet = nullptr;
	uint32_t m_videoStream = -1;
	int64_t m_packetDuration = 0;

public:
	bool openInput(const char* url, FrameParams outputFrameParams, const AVInputFormat* inputFormat = nullptr, AVDictionary* inputOptions = nullptr);
	const AVFrame* readFrame();
	void fastForward(double seconds);
	void close() override;
};




class ScreenRecorder
{
protected:
	const AVFrame* m_frameRGB = nullptr;
	std::mutex m_captureStopMutex;
	bool m_isCaptureStopped = true;
	bool m_isOpen = false;
	std::thread* m_screenCapThread = nullptr;
	VideoReader m_desktop;
	VideoWriter m_videoWriter;

protected:
	void captureScreen();
	virtual void onFrameUpdate();

public:
	bool initCapture(int windowWidth, int windowHeight, int offsetX, int offsetY, int outputFrameWidth, int outputFrameHeight);
	bool startCapture();
	void stopCapture();
	const AVFrame* const getFrameRGB();
	~ScreenRecorder();
};
