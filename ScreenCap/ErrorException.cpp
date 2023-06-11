#include "ErrorException.h"
#include <string>

Error::Error(const char* callingFunction, const char* msg)
	: exception(
		std::string(
			std::string("[ERROR] ") + 
			callingFunction + 
			std::string(" -> ") + 
			msg + '!'
		).c_str()
	)
{
}
