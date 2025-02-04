#define PI_BAUDRATE (230400)

void setup() {
  // put your setup code here, to run once:
  Serial.begin(PI_BAUDRATE);
  Serial1.begin(PI_BAUDRATE);
}

char c;
void loop() {
  // From Pi
  if (Serial1.available()) {
    c = Serial1.read();

    if (c == '\n') {
      Serial.println();
    } else {
      Serial.print(c);
    }
  }

  // To Pi
  if (Serial.available()) {
    c = Serial.read();
    if (c == '\r') {
      Serial.print('\n');
    }
    Serial1.print(c);
  }
}
