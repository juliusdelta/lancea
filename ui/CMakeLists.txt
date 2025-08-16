cmake_minimum_required(VERSION 3.21)
project(lancea-ui LANGUAGES CXX)

find_package(Qt6 6.5 REQUIRED COMPONENTS Quick Qml Gui DBus) # Quick/QML UI + QtDBus

qt_standard_project_setup()

set(CMAKE_AUTOMOC ON)
set(CMAKE_AUTORCC ON)
set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

add_executable(lancea-ui
  src/main.cpp
  src/EngineProxy.h
  src/EngineProxy.cpp
  src/ClipboardProxy.h
  src/AppProxy.h
)

# Bundle QML with the app
qt_add_qml_module(lancea-ui
  URI Lancea
  VERSION 1.0
  QML_FILES
    qml/Main.qml
    # qml/ResultsView.qml
    qml/PreviewPane.qml
)

target_link_libraries(lancea-ui PRIVATE Qt6::Quick Qt6::Qml Qt6::Gui Qt6::DBus)
install(TARGETS lancea-ui)
