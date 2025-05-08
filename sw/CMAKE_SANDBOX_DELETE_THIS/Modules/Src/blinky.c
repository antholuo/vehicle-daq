#include "blinky.h"
#include "gpio.h"

void blinky() {
    while (1) {
        HAL_GPIO_TogglePin(GPIO_LED2_GPIO_Port, GPIO_LED2_Pin);
        HAL_Delay(1000);
    }
}
