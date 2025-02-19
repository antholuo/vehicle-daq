################################################################################
# Automatically-generated file. Do not edit!
# Toolchain: GNU Tools for STM32 (12.3.rel1)
################################################################################

# Add inputs and outputs from these tool invocations to the build variables 
C_SRCS += \
../ASM330/asm330lhh_demo.c \
../ASM330/asm330lhh_reg.c 

OBJS += \
./ASM330/asm330lhh_demo.o \
./ASM330/asm330lhh_reg.o 

C_DEPS += \
./ASM330/asm330lhh_demo.d \
./ASM330/asm330lhh_reg.d 


# Each subdirectory must supply rules for building sources it contributes
ASM330/%.o ASM330/%.su ASM330/%.cyclo: ../ASM330/%.c ASM330/subdir.mk
	arm-none-eabi-gcc "$<" -mcpu=cortex-m0 -std=gnu11 -g3 -DDEBUG -DUSE_HAL_DRIVER -DSTM32F042x6 -c -I../Core/Inc -I../Drivers/STM32F0xx_HAL_Driver/Inc -I../Drivers/STM32F0xx_HAL_Driver/Inc/Legacy -I../Drivers/CMSIS/Device/ST/STM32F0xx/Include -I../Drivers/CMSIS/Include -I"C:/Code/vehicle-daq/sw/nucleo_testy2/ASM330" -O0 -ffunction-sections -fdata-sections -Wall -fstack-usage -fcyclomatic-complexity -MMD -MP -MF"$(@:%.o=%.d)" -MT"$@" --specs=nano.specs -mfloat-abi=soft -mthumb -o "$@"

clean: clean-ASM330

clean-ASM330:
	-$(RM) ./ASM330/asm330lhh_demo.cyclo ./ASM330/asm330lhh_demo.d ./ASM330/asm330lhh_demo.o ./ASM330/asm330lhh_demo.su ./ASM330/asm330lhh_reg.cyclo ./ASM330/asm330lhh_reg.d ./ASM330/asm330lhh_reg.o ./ASM330/asm330lhh_reg.su

.PHONY: clean-ASM330

