#!/usr/bin/python3
from datetime import datetime
import random
import numpy as np




if __name__ == "__main__":
    now = datetime.now()
    dt_string = now.strftime("%d%m%Y%H:%M:%S")
    file_name = f"data_{dt_string}.txt"
    with open(file_name, "w") as fh:
        for l in range(512):
            for s in range(10):
                now = datetime.now()
                dt_string = now.strftime("%d%m%Y%H:%M:%S")
                val = random.randint(50,100)
                shape, scale = 2., 2.  # mean=4, std=2*sqrt(2)
                #sample = np.random.gamma(shape, scale, 1)


                pers = np.arange(1,10000,1)

                # Make each of the last 41 elements 5x more likely
                prob = [1.0]*(len(pers))

                # Normalising to 1.0
                prob /= np.sum(prob)

                sample = np.random.choice(pers, 1, p=prob)
                system = f"starscourge{s}"
                line = f"{dt_string} {int(sample[0])} {system}\n"
                fh.write(line)
