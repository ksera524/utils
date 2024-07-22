import requests
import json
from io import BytesIO

def send_slack_message(token, channel, message):
    """
    Slackにメッセージを送信する関数。

    :param token: Slack APIトークン
    :param channel: メッセージを送信するチャンネルID
    :param message: 送信するメッセージ
    """
    # SlackのトークンとチャンネルIDを設定
    TOKEN = token
    CHANNEL = channel

    headers = {
        "Authorization": f"Bearer {TOKEN}",
        "Content-Type": "application/json"
    }
    data = {
        "channel": CHANNEL,
        "text": message
    }

    response = requests.post("https://slack.com/api/chat.postMessage", headers=headers, data=json.dumps(data))

    if response.status_code == 200:
        print("Message sent successfully")
        return response.json()
    else:
        print("Error:", response.status_code, response.text)
        return None
    
def upload_image_to_slack(token, image, filename):
    """
    画像をSlackにアップロードするための共通関数

    :param token: Slack APIトークン
    :param image: BytesIO形式の画像データ
    :param filename: 画像のファイル名
    :return: (file_id, None) 成功時、(None, error_message) 失敗時
    """
    headers = {'Authorization': f'Bearer {token}'}

    # Step 1: アップロードURLの取得
    response_get = requests.get(
        url='https://slack.com/api/files.getUploadURLExternal',
        headers=headers,
        params={'filename': filename, 'length': len(image.getvalue())}
    )
    
    if response_get.status_code != 200 or not response_get.json()['ok']:
        return None, f"Error getting upload URL: {response_get.status_code} {response_get.text}"
    
    upload_url = response_get.json()['upload_url']
    file_id = response_get.json()['file_id']
    
    # Step 2: 画像のアップロード
    response_post_file = requests.post(
        url=upload_url,
        data=image.getvalue(),
        headers={'Content-Type': 'application/octet-stream'}
    )
    
    if response_post_file.status_code != 200:
        return None, f"Error uploading file: {response_post_file.status_code} {response_post_file.text}"
    
    return file_id, None

def send_single_image_to_slack(token, channel_id, image, filename, title):
    """
    Slackに単一の画像を送信する関数。

    :param token: Slack APIトークン
    :param channel_id: 画像を送信するチャンネルID
    :param image: BytesIO形式の画像データまたはファイルオブジェクト
    :param filename: 画像のファイル名
    :param title: 画像のタイトル
    :return: レスポンスのJSON
    """
    # ファイルオブジェクトの場合、BytesIOに変換
    if hasattr(image, 'read'):
        image_data = BytesIO(image.read())
    else:
        image_data = image

    file_id, error = upload_image_to_slack(token, image_data, filename)
    if error:
        print(error)
        return None

    # Step 3: アップロードの完了
    response_post_channel = requests.post(
        url='https://slack.com/api/files.completeUploadExternal',
        headers={'Content-Type': 'application/json', 'Authorization': f'Bearer {token}'},
        json={
            'files': [{'title': title, 'id': file_id}],
            'channel_id': channel_id
        }
    )
    
    if response_post_channel.status_code == 200:
        print("Image sent successfully")
        return response_post_channel.json()
    else:
        print("Error:", response_post_channel.status_code, response_post_channel.text)
        return None

def send_multiple_images_to_slack(token, channel_id, images):
    """
    Slackに複数の画像を送信する関数。

    :param token: Slack APIトークン
    :param channel_id: 画像を送信するチャンネルID
    :param images: 辞書のリスト。各辞書は {'image': BytesIO or file object, 'filename': str, 'title': str} の形式
    :return: レスポンスのJSON
    """
    files = []
    
    for img_data in images:
        # ファイルオブジェクトの場合、BytesIOに変換
        if hasattr(img_data['image'], 'read'):
            image = BytesIO(img_data['image'].read())
        else:
            image = img_data['image']

        file_id, error = upload_image_to_slack(token, image, img_data['filename'])
        if error:
            print(error)
            continue
        
        files.append({'id': file_id, 'title': img_data['title']})
    
    # Step 3: すべての画像のアップロードの完了
    if files:
        response_post_channel = requests.post(
            url='https://slack.com/api/files.completeUploadExternal',
            headers={'Content-Type': 'application/json', 'Authorization': f'Bearer {token}'},
            json={
                'files': files,
                'channel_id': channel_id
            }
        )
        
        if response_post_channel.status_code == 200 and response_post_channel.json()['ok']:
            print("Images sent successfully")
            return response_post_channel.json()
        else:
            print("Error:", response_post_channel.status_code, response_post_channel.text)
            return None
    else:
        print("No images were successfully prepared for upload")
        return None