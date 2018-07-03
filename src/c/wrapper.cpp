#include "Options.h"
#include "Manager.h"
#include "Driver.h"
#include "Node.h"
#include "Group.h"
#include "Notification.h"
#include "value_classes/ValueStore.h"
#include "value_classes/Value.h"
#include "value_classes/ValueBool.h"
#include "platform/Log.h"
#include "Defs.h"

using namespace OpenZWave;

static Manager::pfnOnNotification_t g_cbk = NULL;
static void *g_ctx = NULL;
static char *g_port = NULL;
static bool g_initialized = false; // we might need an atomic bool here 

struct FFIValueID {
    uint32_t home_id;
    uint8_t node_id;
    ValueID::ValueGenre genre;
    uint8_t command_class_id;
    uint8_t instance;
    uint8_t value_index;
    ValueID::ValueType value_type;
};

extern "C" {
Notification::NotificationType Notification_GetType(Notification const* notification) {
	return notification->GetType();
}

uint8_t Notification_GetNodeId(Notification const* notification) {
	return notification->GetNodeId();
}

uint32_t Notification_GetHomeId(Notification const* notification) {
	return notification->GetHomeId();
}
void Notification_GetValueID(Notification const* notification, FFIValueID *vid) {
	ValueID const & _vid = notification->GetValueID();
	vid->home_id = _vid.GetHomeId();
	vid->node_id = _vid.GetNodeId();
	vid->genre = _vid.GetGenre();
	vid->command_class_id = _vid.GetCommandClassId();
	vid->instance = _vid.GetInstance();
	vid->value_index = _vid.GetIndex();
	vid->value_type = _vid.GetType();
}

void Manager_SwitchAllOff(uint32_t homeId) {
	Manager::Get()->SwitchAllOff(homeId);
}
void Manager_SwitchAllOn(uint32_t homeId) {
	Manager::Get()->SwitchAllOn(homeId);
}

void Manager_TurnNodeOff(uint32_t homeId, uint8_t nodeId) {
	Manager::Get()->SetNodeOff(homeId, nodeId);
}
void Manager_TurnNodeOn(uint32_t homeId, uint8_t nodeId) {
	Manager::Get()->SetNodeOn(homeId, nodeId);
}
void Manager_CancelControllerCommand(uint32_t homeId) {
	Manager::Get()->CancelControllerCommand(homeId);
}
Notification::NotificationCode Notification_GetNotification(Notification const *n) {
	return (Notification::NotificationCode)n->GetNotification();
}

bool Manager_StartInit(const char *port, Manager::pfnOnNotification_t cbk, void* ctx) {
	if (g_initialized) { return false; }

	g_initialized = true;

	Options::Create("/etc/openzwave", "", "");
	Options::Get()->AddOptionInt("SaveLogLevel", LogLevel_None);
	Options::Get()->AddOptionInt("QueueLogLevel", LogLevel_None);
	Options::Get()->AddOptionInt("DumpTrigger", LogLevel_None);
	Options::Get()->AddOptionInt("PollInterval", 500);
	Options::Get()->AddOptionBool("IntervalBetweenPolls", true);
	Options::Get()->AddOptionBool("ValidateValueChanges", true);
	Options::Get()->Lock();

	g_cbk = cbk;
	g_ctx = ctx;
	g_port = strdup(port);

	Manager::Create();
	Manager::Get()->AddWatcher(cbk, ctx);
	Manager::Get()->AddDriver(port);

	return true;
}

char *Manager_GetNodeName(uint32_t homeId, uint8_t nodeId) {
	return strdup(Manager::Get()->GetNodeName(homeId, nodeId).c_str()); // this can probably throw an exception
}

char *Manager_GetNodeManufacturerName(uint32_t homeId, uint8_t nodeId) {
	return strdup(Manager::Get()->GetNodeManufacturerName(homeId, nodeId).c_str()); // this can probably throw an exception
}

char *Manager_GetNodeProductName(uint32_t homeId, uint8_t nodeId) {
	return strdup(Manager::Get()->GetNodeProductName(homeId, nodeId).c_str()); // this can probably throw an exception
}

char *Manager_GetValueLabel(FFIValueID const *v) {
	ValueID vid(v->home_id, v->node_id, v->genre, v->command_class_id, v->instance, v->value_index, v->value_type);
	return strdup(Manager::Get()->GetValueLabel(vid).c_str());
}

char *Manager_GetValueHelp(FFIValueID const *v) {
	ValueID vid(v->home_id, v->node_id, v->genre, v->command_class_id, v->instance, v->value_index, v->value_type);
	return strdup(Manager::Get()->GetValueHelp(vid).c_str());
}

char *Manager_GetValueUnit(FFIValueID const *v) {
	ValueID vid(v->home_id, v->node_id, v->genre, v->command_class_id, v->instance, v->value_index, v->value_type);
	return strdup(Manager::Get()->GetValueUnits(vid).c_str());
}

uint32_t Manager_GetValueMin(FFIValueID const *v) {
	ValueID vid(v->home_id, v->node_id, v->genre, v->command_class_id, v->instance, v->value_index, v->value_type);
	return Manager::Get()->GetValueMin(vid);
}

uint32_t Manager_GetValueMax(FFIValueID const *v) {
	ValueID vid(v->home_id, v->node_id, v->genre, v->command_class_id, v->instance, v->value_index, v->value_type);
	return Manager::Get()->GetValueMax(vid);
}

void Manager_SetValueBool(FFIValueID const *v, bool state) {
	ValueID vid(v->home_id, v->node_id, v->genre, v->command_class_id, v->instance, v->value_index, v->value_type);
	Manager::Get()->SetValue(vid, state);
}
char *Manager_GetValueAsString(FFIValueID const *v) {
	ValueID vid(v->home_id, v->node_id, v->genre, v->command_class_id, v->instance, v->value_index, v->value_type);
    string str;
    Manager::Get()->GetValueAsString(vid, &str);
    return strdup(str.c_str());
}

void Manager_GetDriverStatistics(uint32_t homeId, Driver::DriverData *data) {
	Manager::Get()->GetDriverStatistics(homeId, data);
}

void Manager_Stop() {
	Manager::Get()->RemoveWatcher(g_cbk, g_ctx);
	Manager::Get()->RemoveDriver(g_port);
	Manager::Destroy();
	Options::Destroy();

	free(g_port);
	g_port = NULL;	
	g_initialized = false;
}

} // extern "C"
