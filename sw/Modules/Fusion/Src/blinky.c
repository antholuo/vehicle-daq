#include "blinky.h"

#include "gpio.h"

void blinky() {
  for (uint8_t i = 0; i < 10; ++i) {
    HAL_GPIO_TogglePin(GPIO_LED2_GPIO_Port, GPIO_LED2_Pin);
    HAL_Delay(1000);
  }
}
