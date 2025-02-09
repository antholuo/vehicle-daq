################################################################################
# Automatically-generated file. Do not edit!
# Toolchain: GNU Tools for STM32 (12.3.rel1)
################################################################################

# Add inputs and outputs from these tool invocations to the build variables 
C_SRCS += \
../Drivers/BSP/Components/asm330lhh/asm330lhh.c \
../Drivers/BSP/Components/asm330lhh/asm330lhh_reg.c 

OBJS += \
./Drivers/BSP/Components/asm330lhh/asm330lhh.o \
./Drivers/BSP/Components/asm330lhh/asm330lhh_reg.o 

C_DEPS += \
./Drivers/BSP/Components/asm330lhh/asm330lhh.d \
./Drivers/BSP/Components/asm330lhh/asm330lhh_reg.d 


# Each subdirectory must supply rules for building sources it contributes
Drivers/BSP/Components/asm330lhh/%.o Drivers/BSP/Components/asm330lhh/%.su Drivers/BSP/Components/asm330lhh/%.cyclo: ../Drivers/BSP/Components/asm330lhh/%.c Drivers/BSP/Components/asm330lhh/subdir.mk
	arm-none-eabi-gcc "$<" -mcpu=cortex-m0 -std=gnu11 -DUSE_HAL_DRIVER -DSTM32F042x6 -c -I../Core/Inc -I../Drivers/STM32F0xx_HAL_Driver/Inc -I../Drivers/STM32F0xx_HAL_Driver/Inc/Legacy -I../Drivers/CMSIS/Device/ST/STM32F0xx/Include -I../Drivers/CMSIS/Include -I../Drivers/BSP/Components/asm330lhh -I../X-CUBE-MEMS1/Target -I../Middlewares/ST/STM32_MotionFX_Library/Inc -Os -ffunction-sections -fdata-sections -Wall -fstack-usage -fcyclomatic-complexity -MMD -MP -MF"$(@:%.o=%.d)" -MT"$@" --specs=nano.specs -mfloat-abi=soft -mthumb -o "$@"

clean: clean-Drivers-2f-BSP-2f-Components-2f-asm330lhh

clean-Drivers-2f-BSP-2f-Components-2f-asm330lhh:
	-$(RM) ./Drivers/BSP/Components/asm330lhh/asm330lhh.cyclo ./Drivers/BSP/Components/asm330lhh/asm330lhh.d ./Drivers/BSP/Components/asm330lhh/asm330lhh.o ./Drivers/BSP/Components/asm330lhh/asm330lhh.su ./Drivers/BSP/Components/asm330lhh/asm330lhh_reg.cyclo ./Drivers/BSP/Components/asm330lhh/asm330lhh_reg.d ./Drivers/BSP/Components/asm330lhh/asm330lhh_reg.o ./Drivers/BSP/Components/asm330lhh/asm330lhh_reg.su

.PHONY: clean-Drivers-2f-BSP-2f-Components-2f-asm330lhh

