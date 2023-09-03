from typing import List


def get_message_from_addition(num1: int, num2: int, message: str = "oi") -> str:
    return f"{message}: {num1 + num2}"


def function_without_return_type(x: str, y: int, a, b: list):
    return "oi"


def hey_jude(num1, num2: int, message="oi") -> str:
    return f"{message}: {num1 + num2}"


def main():
    message_from_addition = get_message_from_addition(3, 1, "Addition")
    print(message_from_addition)


class Hey:
    def ho(self, p, q: int) -> int:
        return q


if __name__ == "__main__":
    main()
