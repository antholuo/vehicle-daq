cmake_minimum_required(VERSION 3.5)
project(sensor_board_rev0 C ASM) # no CXX

set (ELF_NAME ${PROJECT_NAME}.elf)
set (HEX_NAME ${PROJECT_NAME}.hex)
set (BIN_NAME ${PROJECT_NAME}.bin)

# Set project root to "sw"
set(PROJECT_ROOT_DIR ${CMAKE_CURRENT_SOURCE_DIR}/../..)

# Set project source dir to "sensor_board_rev0"
set(PROJECT_SOURCE_DIR ${CMAKE_CURRENT_SOURCE_DIR})

### Includes ###
include_directories(
  # Default STM32 HAL Libraries
  ${CMAKE_CURRENT_SOURCE_DIR}/Core/Inc
  ${CMAKE_CURRENT_SOURCE_DIR}/Drivers/CMSIS/Include
  ${CMAKE_CURRENT_SOURCE_DIR}/Drivers/CMSIS/Device/ST/STM32F0xx/Include
  ${CMAKE_CURRENT_SOURCE_DIR}/Drivers/STM32F0xx_HAL_Driver/Inc

  # Project specific includes
  ${PROJECT_ROOT_DIR}/Modules/Fusion/Inc # Data Fusion
  ${PROJECT_ROOT_DIR}/Drivers/Imu/Inc    # Imu Driver
  ${PROJECT_ROOT_DIR}/Drivers/Gps/Inc    # Gps Driver

  # OLD PROJECT INCLUDES
  # ${PROJECT_SOURCE_DIR}/Modules
  # ${PROJECT_SOURCE_DIR}/Modules/canard_files/Inc
)

### Sources ### # Eventually - we'll want specific control on our source files
set(MODULES_DIR ${PROJECT_ROOT_DIR}/Modules)
set(MODULES_C_SRCS ${MODULES_DIR}/**/Src/*.c)

set(DRIVERS_DIR ${PROJECT_ROOT_DIR}/Drivers)
set(DRIVERS_C_SRCS ${DRIVERS_DIR}/**/Src/*.c)

# set(OLD_C_SRCS ${PROJECT_SOURCE_DIR}/Modules/*.c)
# set(OLD_CANARD_SRCS ${PROJECT_SOURCE_DIR}/Modules/canard_files/Src/*.c)

file(GLOB_RECURSE C_SOURCES ${CMAKE_CURRENT_SOURCE_DIR}/Core/*.c
                            ${CMAKE_CURRENT_SOURCE_DIR}/Drivers/STM32F0xx_HAL_Driver/Src/*.c
                            ${CMAKE_CURRENT_SOURCE_DIR}/Drivers/CMSIS/*.c
                            ${MODULES_C_SRCS}
                            # ${OLD_C_SRCS}
                            # ${OLD_CANARD_SRCS}
)


set(STARTUP_ASM ${CMAKE_CURRENT_SOURCE_DIR}/Core/Startup/startup_stm32f042k6tx.s)
set(LINKER_SCRIPT ${CMAKE_CURRENT_SOURCE_DIR}/STM32F042K6TX_FLASH.ld)

add_executable(${ELF_NAME} ${C_SOURCES} ${STARTUP_ASM})

# We don't have an STM32 HAL Library
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
