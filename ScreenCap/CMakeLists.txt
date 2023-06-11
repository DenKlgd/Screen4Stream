cmake_minimum_required (VERSION 3.8)
project("ScreenCap")

set(FFMPEG_DIR "E:/FFMPEG/ffmpeg-master-latest-win64-gpl-shared")
#set(CMAKE_WINDOWS_EXPORT_ALL_SYMBOLS ON)

SET (FFMPEG_LIBS
  "${FFMPEG_DIR}/lib/avcodec.lib"
  "${FFMPEG_DIR}/lib/avdevice.lib"
  "${FFMPEG_DIR}/lib/avfilter.lib"
  "${FFMPEG_DIR}/lib/avformat.lib"
  "${FFMPEG_DIR}/lib/avutil.lib"
  "${FFMPEG_DIR}/lib/swresample.lib"
  "${FFMPEG_DIR}/lib/swscale.lib"
  )

add_library(ScreenCap STATIC "ScreenCap.cpp" "ScreenCap.h" "ErrorException.h" "ErrorException.cpp")

target_include_directories(ScreenCap PUBLIC "${FFMPEG_DIR}/include")

target_link_libraries(ScreenCap ${FFMPEG_LIBS})
