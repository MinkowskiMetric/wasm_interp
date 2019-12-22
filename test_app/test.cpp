#include <stdio.h>

int test_fib(int n)
{
	// This is deliberately rubbish
	if (n < 2)
	{
		return 1;
	}
	else
	{
		return test_fib(n-1) + test_fib(n-2);
	}
}

int main()
{
	for (int a = 0; a < 17; ++a)
	{
		printf("bogan %d\n", test_fib(a));
	}
}

