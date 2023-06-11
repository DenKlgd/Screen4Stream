cmake_minimum_required (VERSION 3.16)

set(CMAKE_AUTOMOC ON)
set(CMAKE_BUILD_TYPE Debug)

find_package(Qt6 6.2 COMPONENTS Quick Gui Network REQUIRED)
include_directories("../ScreenCap/")

set(SOURCES main.cpp
    framerenderer.h
    framerenderer.cpp
    screenrecordercustom.h
    screenrecordercustom.cpp
    Frame.h
    Frame.cpp
    UdpSocketCustom.h
    UdpSocketCustom.cpp
)

#add_executable (${CMAKE_PROJECT_NAME} WIN32 ${SOURCES})
add_executable (${CMAKE_PROJECT_NAME} ${SOURCES})

qt_add_qml_module(${CMAKE_PROJECT_NAME}
    URI QmlWindow
    VERSION 1.0
    QML_FILES main.qml 
)


target_link_libraries(${CMAKE_PROJECT_NAME}
    PRIVATE 
    Qt6::Quick 
    Qt6::Gui
    Qt6::Qml
    Qt6::Network
    )

target_link_libraries(${CMAKE_PROJECT_NAME} PUBLIC ScreenCap)
