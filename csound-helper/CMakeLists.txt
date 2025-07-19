cmake_minimum_required(VERSION 3.16)
project(qlaunchpad LANGUAGES C)

list(APPEND CMAKE_MODULE_PATH "/usr/share/cmake/Csound")
set(CMAKE_SUPPRESS_DEVELOPER_WARNINGS 1)
find_package(Csound REQUIRED)

message("!${CSOUND_FOUND}")
message("!${CSOUND_INCLUDE_DIRS}")
message("!${CSOUND_LIBRARIES}")
