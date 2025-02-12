/*
 * asm330lhh_demo.c
 *
 * Created on: 2025-FEB-11 by Anni
 * This is a file which we will use to demonstrate the functionality of the ASM330LHH sensor
 */

#include "asm330lhh_demo.h"
#include "gpio.h"

void startAsm330lhhDemo(void)
{
    // This is a placeholder for the demo code
    while (1) {
        HAL_GPIO_TogglePin(GPIO_LED_GPIO_Port, GPIO_LED_Pin);

        HAL_Delay(1000);
    }
}