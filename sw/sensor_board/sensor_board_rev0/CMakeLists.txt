cmake_minimum_required(VERSION 3.5)
project(sensor_board_rev0 C ASM) # no CXX

set (ELF_NAME ${PROJECT_NAME}.elf)
set (HEX_NAME ${PROJECT_NAME}.hex)
set (BIN_NAME ${PROJECT_NAME}.bin)

# Set project root to "sw"
set(PROJECT_ROOT_DIR ${CMAKE_CURRENT_SOURCE_DIR}/../..)

# Set project source dir to "sensor_board_rev0"
set(PROJECT_SOURCE_DIR ${CMAKE_CURRENT_SOURCE_DIR})

# Disabling the can reception handle for now... for the sake of binary size
add_definitions(-DSENSOR_BOARD_CAN_RECEPTION_DISABLE)

# Provide default value
option(NODE_ID "Board's CAN Node ID" "20")
option(BOARD_INIT_DELAY_MS "Board initialization delay in ms" "0")

# Pass it as a compile definition
add_compile_definitions(NODE_ID=0x${NODE_ID})
add_compile_definitions(BOARD_INIT_DELAY_MS=${BOARD_INIT_DELAY_MS})

### Includes ###
include_directories(
  # Default STM32 HAL Libraries
  ${CMAKE_CURRENT_SOURCE_DIR}/Core/Inc
  ${CMAKE_CURRENT_SOURCE_DIR}/Drivers/CMSIS/Include
  ${CMAKE_CURRENT_SOURCE_DIR}/Drivers/CMSIS/Device/ST/STM32F0xx/Include
  ${CMAKE_CURRENT_SOURCE_DIR}/Drivers/STM32F0xx_HAL_Driver/Inc

  # Project specific includes
  ${PROJECT_ROOT_DIR}/Modules/Fusion/Inc # Data Fusion
  ${PROJECT_ROOT_DIR}/Modules/Dsdlc_generated/Inc # DroneCAN message Libraries
  ${PROJECT_ROOT_DIR}/Modules/Comms/Inc # Inter-board Communications
  ${PROJECT_ROOT_DIR}/Modules/Canard/Inc # Canard Library
  ${PROJECT_ROOT_DIR}/Drivers/Imu/Inc    # Imu Driver
  ${PROJECT_ROOT_DIR}/Drivers/Gps/Inc    # Gps Driver

  ${PROJECT_ROOT_DIR}/Protos/Generated/Inc    # Generated Protobuf Library
  ${PROJECT_ROOT_DIR}/Protos/Pb_libs/Inc    # Protobuf Library
)

### Sources ### # Eventually - we'll want specific control on our source files
set(MODULES_DIR ${PROJECT_ROOT_DIR}/Modules)
set(MODULES_C_SRCS
  ${MODULES_DIR}/Fusion/Src/sensor_board.c
  ${MODULES_DIR}/Dsdlc_generated/Src/*.c
  ${MODULES_DIR}/Comms/Src/*.c
  ${MODULES_DIR}/Canard/Src/*.c
)

set(PROTOBUF_DIR ${PROJECT_ROOT_DIR}/Protos)
set(PROTOBUF_C_SRCS ${PROTOBUF_DIR}/**/Src/*.c)

set(DRIVERS_DIR ${PROJECT_ROOT_DIR}/Drivers)
set(DRIVERS_C_SRCS
  ${DRIVERS_DIR}/Imu/Src/*.c
  ${DRIVERS_DIR}/Gps/Src/*.c
)

file(GLOB_RECURSE C_SOURCES ${CMAKE_CURRENT_SOURCE_DIR}/Core/*.c
                            ${CMAKE_CURRENT_SOURCE_DIR}/Drivers/STM32F0xx_HAL_Driver/Src/*.c
                            ${CMAKE_CURRENT_SOURCE_DIR}/Drivers/CMSIS/*.c
                            ${MODULES_C_SRCS}
                            ${DRIVERS_C_SRCS}
)


set(STARTUP_ASM ${CMAKE_CURRENT_SOURCE_DIR}/Core/Startup/startup_stm32f042k6tx.s)
set(LINKER_SCRIPT ${CMAKE_CURRENT_SOURCE_DIR}/STM32F042K6TX_FLASH.ld)

add_executable(${ELF_NAME} ${C_SOURCES} ${STARTUP_ASM})

# We don't have an STM32 HAL Library
# target_link_libraries(${ELF_NAME} STM32_HAL)

# Add project-specific linker flags (.ld script, .map file)
set(CMAKE_EXE_LINKER_FLAGS "${CMAKE_EXE_LINKER_FLAGS} -T${LINKER_SCRIPT} -Wl,-Map=${PROJECT_BINARY_DIR}/${PROJECT_NAME}.map,--cref")

# Enable optimization to this project
set(CMAKE_C_FLAGS_RELEASE "-O2 -DNDEBUG")
set(CMAKE_C_FLAGS_DEBUG   "-O1 -g")

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
