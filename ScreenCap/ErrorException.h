#pragma once
#include <exception>

class Error : public std::exception
{
public:
	Error(const char* callingFunction, const char* msg);
};

#define ERROR(msg) \
	Error(__FUNCTION__, ((msg)))
