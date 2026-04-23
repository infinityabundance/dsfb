cmake_minimum_required(VERSION 3.16)
project(gr-dsfb CXX)

# ─── GNU Radio dependency ─────────────────────────────────────────────────────
find_package(Gnuradio "3.10" REQUIRED COMPONENTS runtime)

# ─── Rust library (pre-built via cargo) ──────────────────────────────────────
# The Rust crate exports a C-ABI shared library (libdsfb_rf.so) which the
# Python block loads via ctypes.  Build it first:
#
#   cd <repo>/dsfb-rf && cargo build --release --features std
#
# The resulting library lands in:
#   <repo>/dsfb-rf/target/release/libdsfb_rf.so   (Linux/macOS)
#   <repo>/dsfb-rf/target/release/dsfb_rf.dll     (Windows)
#
if(NOT DEFINED DSFB_RF_LIB)
    set(DSFB_RF_LIB "${CMAKE_CURRENT_SOURCE_DIR}/../dsfb-rf/target/release/libdsfb_rf.so")
endif()

message(STATUS "gr-dsfb: Rust library path: ${DSFB_RF_LIB}")

if(NOT EXISTS "${DSFB_RF_LIB}")
    message(WARNING
        "gr-dsfb: Rust library not found at ${DSFB_RF_LIB}.\n"
        "  Build it with: cargo build --release --features std\n"
        "  Then re-run CMake with -DDSFB_RF_LIB=<path>")
endif()

# ─── Installation prefix ──────────────────────────────────────────────────────
if(NOT CMAKE_INSTALL_PREFIX)
    set(CMAKE_INSTALL_PREFIX "/usr/local")
endif()

# ─── Submodules ──────────────────────────────────────────────────────────────
add_subdirectory(python)
add_subdirectory(grc)

# ─── Install Rust library alongside Python module ─────────────────────────────
install(FILES "${DSFB_RF_LIB}"
    DESTINATION "${CMAKE_INSTALL_PREFIX}/lib"
    OPTIONAL)
