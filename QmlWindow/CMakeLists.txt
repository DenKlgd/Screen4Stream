cmake_minimum_required (VERSION 3.16)

set(CMAKE_AUTOMOC ON)
set(CMAKE_BUILD_TYPE Debug)

find_package(Qt6 6.2 COMPONENTS Quick Multimedia Gui REQUIRED)
include_directories("../ScreenCap/")

set(SOURCES main.cpp
    framerenderer.h
    framerenderer.cpp
    screenrecordercustom.h
    screenrecordercustom.cpp
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
    Qt6::Multimedia 
    Qt6::Gui
    Qt6::Qml
    )

target_link_libraries(${CMAKE_PROJECT_NAME} PUBLIC ScreenCap)
