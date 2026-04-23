find_package(Python3 REQUIRED COMPONENTS Interpreter)

# Install the gr_dsfb Python package
execute_process(
    COMMAND "${Python3_EXECUTABLE}" -c "import site; print(site.getsitepackages()[0])"
    OUTPUT_VARIABLE PYTHON_SITE_PACKAGES
    OUTPUT_STRIP_TRAILING_WHITESPACE)

install(DIRECTORY "${CMAKE_CURRENT_SOURCE_DIR}/gr_dsfb"
    DESTINATION "${PYTHON_SITE_PACKAGES}"
    FILES_MATCHING PATTERN "*.py")
