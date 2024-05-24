/* tslint:disable */
/* eslint-disable */
/**
 * server
 * No description provided (generated by Openapi Generator https://github.com/openapitools/openapi-generator)
 *
 * The version of the OpenAPI document: 0.1.0
 * Contact: booiris02@gmail.com
 *
 * NOTE: This class is auto generated by OpenAPI Generator (https://openapi-generator.tech).
 * https://openapi-generator.tech
 * Do not edit the class manually.
 */

import { mapValues } from '../runtime';
/**
 * 
 * @export
 * @interface LoginReq
 */
export interface LoginReq {
    /**
     * 
     * @type {string}
     * @memberof LoginReq
     */
    email: string;
    /**
     * 
     * @type {string}
     * @memberof LoginReq
     */
    password: string;
}

/**
 * Check if a given object implements the LoginReq interface.
 */
export function instanceOfLoginReq(value: object): value is LoginReq {
    if (!('email' in value) || value['email'] === undefined) return false;
    if (!('password' in value) || value['password'] === undefined) return false;
    return true;
}

export function LoginReqFromJSON(json: any): LoginReq {
    return LoginReqFromJSONTyped(json, false);
}

export function LoginReqFromJSONTyped(json: any, ignoreDiscriminator: boolean): LoginReq {
    if (json == null) {
        return json;
    }
    return {
        
        'email': json['email'],
        'password': json['password'],
    };
}

export function LoginReqToJSON(value?: LoginReq | null): any {
    if (value == null) {
        return value;
    }
    return {
        
        'email': value['email'],
        'password': value['password'],
    };
}
