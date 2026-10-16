// Anisette of the Mac itself, from the system's own frameworks (AOSKit for the one-time password and machine id, AuthKit
// for the device identifiers), the way AltServer gets it. Apple sees a real Mac, not a remote anisette server.
#import <Foundation/Foundation.h>
#import <dlfcn.h>
#import <objc/message.h>

static int copy_out(id value, char *out, size_t cap) {
    NSString *s = [value description];
    if (s == nil || cap == 0) return 0;
    const char *c = s.UTF8String;
    if (c == NULL || strlen(c) >= cap) return 0;
    strcpy(out, c);
    return 1;
}

static id send0(id target, NSString *selector) {
    SEL sel = NSSelectorFromString(selector);
    if (target == nil || ![target respondsToSelector:sel]) return nil;
    return ((id (*)(id, SEL))objc_msgSend)(target, sel);
}

// Fills the five strings (each `cap` bytes); returns 1 on success.
int isideload_mac_anisette(char *otp, char *machine_id, char *device_id, char *description, char *serial, size_t cap) {
    @autoreleasepool {
        if (!dlopen("/System/Library/PrivateFrameworks/AOSKit.framework/AOSKit", RTLD_NOW)) return 0;
        if (!dlopen("/System/Library/PrivateFrameworks/AuthKit.framework/AuthKit", RTLD_NOW)) return 0;
        Class aos = NSClassFromString(@"AOSUtilities");
        SEL otpSel = NSSelectorFromString(@"retrieveOTPHeadersForDSID:");
        if (aos == nil || ![aos respondsToSelector:otpSel]) return 0;
        NSDictionary *headers = ((id (*)(id, SEL, id))objc_msgSend)(aos, otpSel, @"-2");
        id device = send0(NSClassFromString(@"AKDevice"), @"currentDevice");
        return copy_out(headers[@"X-Apple-MD"], otp, cap)
            && copy_out(headers[@"X-Apple-MD-M"], machine_id, cap)
            && copy_out(send0(device, @"uniqueDeviceIdentifier"), device_id, cap)
            && copy_out(send0(device, @"serverFriendlyDescription"), description, cap)
            && copy_out(send0(device, @"serialNumber"), serial, cap);
    }
}
