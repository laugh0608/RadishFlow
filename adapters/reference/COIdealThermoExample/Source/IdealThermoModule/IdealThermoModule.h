
//function declarations
string GetUserDataPath();
string GetDataPath();
bool ReadLine(FILE *f,string &line);
void ListFiles(const char *folder,const char *ext,vector<string> &fileNames);
string ErrorString(int errCode);