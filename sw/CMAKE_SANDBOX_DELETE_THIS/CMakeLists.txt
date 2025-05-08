cmake_minimum_required(VERSION 3.5)
project(nucleo_cmake_test C ASM)

set (ELF_NAME ${PROJECT_NAME}.elf)
set (HEX_NAME ${PROJECT_NAME}.hex)
set (BIN_NAME ${PROJECT_NAME}.bin)

set(CMAKE_SYSTEM_NAME Generic)
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

# set the project root directory to "sw"
set(PROJECT_ROOT_DIR ${CMAKE_CURRENT_SOURCE_DIR})
set(PROJECT_SOURCE_DIR ${CMAKE_CURRENT_SOURCE_DIR}/nucleo_cmake_test)

include_directories(
    # Default STM32 HAL libraries
    ${PROJECT_ROOT_DIR}/nucleo_cmake_test
    ${PROJECT_ROOT_DIR}/nucleo_cmake_test/Core/Inc
    ${PROJECT_ROOT_DIR}/nucleo_cmake_test/Drivers/STM32F0xx_HAL_Driver/Inc
    ${PROJECT_ROOT_DIR}/nucleo_cmake_test/Drivers/CMSIS/Include
    ${PROJECT_ROOT_DIR}/nucleo_cmake_test/Drivers/CMSIS/Device/ST/STM32F0xx/Include

    # Project specific includes
    ${PROJECT_ROOT_DIR}/Modules/Inc
)

## Module Sources
set(MODULES_DIR ${PROJECT_ROOT_DIR}/Modules)
set(MODULES_C_SRCS ${MODULES_DIR}/Src/*.c)

file(GLOB_RECURSE C_SOURCES ${PROJECT_ROOT_DIR}/nucleo_cmake_test/*.c
                            ${PROJECT_ROOT_DIR}/nucleo_cmake_test/Core/*.c
                            ${PROJECT_ROOT_DIR}/nucleo_cmake_test/Drivers/STM32F0xx_HAL_Driver/Src/*.c
                            ${PROJECT_ROOT_DIR}/nucleo_cmake_test/Drivers/CMSIS/*.c
                            ${MODULES_C_SRCS}
)


# set(STARTUP_ASM startup_stm32f042k6tx.s)
# set(LINKER_SCRIPT ${PROJECT_SOURCE_DIR}/STM32F042RCTx_FLASH.ld)

set(STARTUP_ASM ${PROJECT_ROOT_DIR}/nucleo_cmake_test/Core/Startup/startup_stm32f042k6tx.s)
set(LINKER_SCRIPT ${PROJECT_ROOT_DIR}/nucleo_cmake_test/STM32F042K6TX_FLASH.ld)

add_executable(${ELF_NAME} ${C_SOURCES} ${STARTUP_ASM})

# We don't have an STM32 HAL
# target_link_libraries(${ELF_NAME} STM32_HAL)

# Add project-specific linker flags (.ld script, .map file)
set(CMAKE_EXE_LINKER_FLAGS "${CMAKE_EXE_LINKER_FLAGS} -T${LINKER_SCRIPT} -Wl,-Map=${PROJECT_BINARY_DIR}/${PROJECT_NAME}.map,--cref")

set(BIN_FILE ${PROJECT_BINARY_DIR}/${PROJECT_NAME}.bin)

# objcopy the .elf file to .bin file for programming
add_custom_target("${PROJECT_NAME}.bin" ALL
  COMMAND ${CMAKE_OBJCOPY} -Obinary $<TARGET_FILE:${ELF_NAME}> ${BIN_FILE}
  DEPENDS ${ELF_NAME}
)

# Print size information after compiling
add_custom_command(TARGET ${ELF_NAME}
  POST_BUILD
  COMMENT "Binary size information:"
  COMMAND ${CMAKE_SIZE} $<TARGET_FILE:${ELF_NAME}>
)

# the following is windows only
set(INSTALL_CMD "ST-LINK_CLI")
set(INSTALL_OPTS "-c SWD -P \"${BIN_FILE}\" 0x08000000 -NoPrompt -Rst -Run")

install(CODE
  "execute_process(
    COMMAND ${INSTALL_CMD} ${INSTALL_OPTS}
  )"
)

message("CMAKE_BUILD_TYPE = ${CMAKE_BUILD_TYPE}")

IF(CMAKE_BUILD_TYPE MATCHES Debug)
    message("Debug build.")
ELSEIF(CMAKE_BUILD_TYPE MATCHES Release)
    message("Release build.")
ELSE()
    message("Some other build type.")
ENDIF()

add_custom_target(UPLOAD
        arm-none-eabi-gdb -iex "target remote tcp:127.0.0.1:3333"
        -iex "monitor program $<TARGET_FILE:${PROJECT_NAME}.elf>"
        -iex "monitor reset init"
        -iex "disconnect" -iex "quit")
