# Cross-compile Windows x64 DLLs on Linux with clang-cl + lld-link against the
# MSVC CRT and Windows SDK unpacked by xwin (`xwin splat`).
# Usage: cmake -G Ninja -DCMAKE_TOOLCHAIN_FILE=cmake/clang-cl-xwin.cmake -DXWIN_DIR=<splat dir> ...

set(CMAKE_SYSTEM_NAME Windows)
set(CMAKE_SYSTEM_VERSION 10.0)
set(CMAKE_SYSTEM_PROCESSOR AMD64)

if(NOT XWIN_DIR)
  set(XWIN_DIR "$ENV{XWIN_DIR}")
endif()
if(NOT XWIN_DIR)
  set(XWIN_DIR "$ENV{HOME}/.local/share/flat-to-3d/xwin")
endif()
if(NOT EXISTS "${XWIN_DIR}/crt/include")
  message(FATAL_ERROR "xwin splat not found at ${XWIN_DIR} (set XWIN_DIR)")
endif()
# Try-compile projects need the same value.
list(APPEND CMAKE_TRY_COMPILE_PLATFORM_VARIABLES XWIN_DIR)

set(CMAKE_C_COMPILER clang-cl)
set(CMAKE_CXX_COMPILER clang-cl)
set(CMAKE_LINKER lld-link)
set(CMAKE_RC_COMPILER llvm-rc)
set(CMAKE_MT llvm-mt)
set(CMAKE_AR llvm-lib)

set(_xwin_flags
    "--target=x86_64-pc-windows-msvc -fuse-ld=lld-link"
    "/imsvc${XWIN_DIR}/crt/include"
    "/imsvc${XWIN_DIR}/sdk/include/ucrt"
    "/imsvc${XWIN_DIR}/sdk/include/um"
    "/imsvc${XWIN_DIR}/sdk/include/shared"
    "/imsvc${XWIN_DIR}/sdk/include/winrt")
string(JOIN " " _xwin_flags ${_xwin_flags})
set(CMAKE_C_FLAGS_INIT "${_xwin_flags}")
set(CMAKE_CXX_FLAGS_INIT "${_xwin_flags}")

set(_xwin_libs
    "/libpath:${XWIN_DIR}/crt/lib/x86_64"
    "/libpath:${XWIN_DIR}/sdk/lib/um/x86_64"
    "/libpath:${XWIN_DIR}/sdk/lib/ucrt/x86_64")
string(JOIN " " _xwin_libs ${_xwin_libs})
set(CMAKE_EXE_LINKER_FLAGS_INIT "${_xwin_libs}")
set(CMAKE_SHARED_LINKER_FLAGS_INIT "${_xwin_libs}")
set(CMAKE_MODULE_LINKER_FLAGS_INIT "${_xwin_libs}")

# llvm-rc needs the SDK headers too (windows.h for .rc files).
set(CMAKE_RC_FLAGS_INIT "/I${XWIN_DIR}/sdk/include/um /I${XWIN_DIR}/sdk/include/shared /I${XWIN_DIR}/crt/include /I${XWIN_DIR}/sdk/include/ucrt")

# xwin ships only the release CRT (no msvcrtd.lib), so compiler checks must not link Debug.
set(CMAKE_TRY_COMPILE_CONFIGURATION Release)
set(CMAKE_MSVC_RUNTIME_LIBRARY MultiThreadedDLL)

set(CMAKE_FIND_ROOT_PATH_MODE_PROGRAM NEVER)
set(CMAKE_FIND_ROOT_PATH_MODE_LIBRARY ONLY)
set(CMAKE_FIND_ROOT_PATH_MODE_INCLUDE ONLY)
