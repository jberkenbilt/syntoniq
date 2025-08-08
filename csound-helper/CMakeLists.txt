cmake_minimum_required(VERSION 3.16)
project(qlaunchpad LANGUAGES C)

list(APPEND CMAKE_MODULE_PATH "/usr/share/cmake/Csound" "/opt/homebrew/share/cmake/Csound")
set(CMAKE_SUPPRESS_DEVELOPER_WARNINGS 1)
find_package(Csound)

if(NOT(CSOUND_FOUND STREQUAL "TRUE"))
  # Special cases for csound on mac
  if(EXISTS "/Library/Frameworks/CsoundLib64.framework/Headers")
    set(CSOUND_FOUND "TRUE")
    set(CSOUND_INCLUDE_DIRS "/Library/Frameworks/CsoundLib64.framework/Headers")
    set(CSOUND_LIBRARIES "framework=/Library/Frameworks/framework=CsoundLib64")
  elseif(EXISTS "/opt/homebrew/Frameworks/CsoundLib64.framework")
    set(CSOUND_FOUND "TRUE")
    set(CSOUND_INCLUDE_DIRS "/opt/homebrew/Frameworks/CsoundLib64.framework/Headers")
    set(CSOUND_LIBRARIES "framework=/opt/homebrew/Frameworks/framework=CsoundLib64")
  endif()
endif()

message("!${CSOUND_FOUND}")
message("!${CSOUND_INCLUDE_DIRS}")
message("!${CSOUND_LIBRARIES}")
