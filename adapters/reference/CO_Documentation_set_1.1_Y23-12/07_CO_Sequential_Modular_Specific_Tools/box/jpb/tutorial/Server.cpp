// Version Classique

#include "GATComponent.h"
#include <fstream.h>

int main(int argc, char* const* argv)
{
  try {
    // Initialize the ORB and BOA
    CORBA::ORB_var orb = CORBA::ORB_init(argc, argv);
    CORBA::BOA_var boa = orb->BOA_init(argc, argv);

    //Create a GATComponent Object
	GATComponent * gat= new GATComponent("CO Prototype");

	//Export the newly created object
	boa->obj_is_ready(gat);
	cout<<"======== New ORB Object : ICapeNumericGATComponent CO Prototype ========"<<endl;
	cout<<endl;

	//Fill the Trader file
	ofstream outfile;
	outfile.open("Trader.txt", ios::out|ios::trunc);
	if(outfile){
		outfile << orb->object_to_string(gat) << endl;
		outfile.close();
	}else cout<<" Error File"<<endl;

    //Wait for incoming requests
    boa->impl_is_ready();

  } catch(const CORBA::Exception& e) {
    cerr << e << endl;
    return(1);
  }
  return(0);
}
