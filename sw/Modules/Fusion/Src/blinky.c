#include "blinky.h"
#include "gpio.h"
#include "tim.h"

/* extern TIM_HandleTypeDef htim3;		// GPIO LED (1 & 2) */

void blinky() {
	/* TIM3->CCR1 = TIM3->ARR / 2; */
	/* HAL_TIM_PWM_Start(&htim3, TIM_CHANNEL_1); // ch1 = led1 */
    for (uint8_t i = 0; i < 10; ++i) {
		    /* TIM3->ARR = TIM3->ARR / 10; */
		    /* TIM3->CCR1 = TIM3->ARR / 2; */
        HAL_GPIO_TogglePin(GPIO_LED2_GPIO_Port, GPIO_LED2_Pin);
        HAL_Delay(100);
    }
}

